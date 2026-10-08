#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as AddressTestUtils;
use soroban_sdk::{token, Address, Env};

fn setup(mock_all_auths: bool) -> (Env, Address, Address, Address) {
    let env = Env::default();
    if mock_all_auths {
        env.mock_all_auths();
    }

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token_address = sac.address();

    if mock_all_auths {
        token::StellarAssetClient::new(&env, &token_address).mint(&user, &10_000_000);
    }

    let contract_id = env.register(EscaleraAhorro, ());
    EscaleraAhorroClient::new(&env, &contract_id).initialize(&token_address);
    (env, contract_id, user, token_address)
}

fn preparar_usuario(
    client: &EscaleraAhorroClient,
    user: &Address,
    meta_inicial: i128,
    meta_bps: u32,
    colchon_bps: u32,
    primer_paso_bps: u32,
) {
    client.abrir_meta(user, &meta_inicial);
    client.configurar_reparto(user, &meta_bps, &colchon_bps, &primer_paso_bps);
}

#[test]
fn deposito_cruza_meta_y_avanza_un_nivel() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 100, 10_000, 0, 0);
    client.depositar(&user, &60);
    client.depositar(&user, &40);

    let estado = client.ver_estado(&user);
    assert_eq!(estado.nivel, 1);
    assert_eq!(estado.insignias, 1);
    assert_eq!(estado.meta_actual, 170);
    assert_eq!(estado.saldo_meta, 100);
    assert_eq!(estado.saldo_colchon, 0);
    assert_eq!(estado.saldo_primer_paso, 0);
}

#[test]
fn acceso_respeta_limite_y_tope_porcentual() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 100, 10_000, 0, 0);
    client.depositar(&user, &100);
    assert_eq!(client.acceso_desbloqueado(&user), 10);
    assert!(client.try_retirar_desbloqueado(&user, &11).is_err());
    client.retirar_desbloqueado(&user, &10);

    client.depositar(&user, &80);
    client.depositar(&user, &119);
    client.depositar(&user, &202);
    client.depositar(&user, &343);
    assert_eq!(client.ver_estado(&user).nivel, 5);
    assert_eq!(client.acceso_desbloqueado(&user), 417);
}

#[test]
fn no_permite_abrir_dos_metas_simultaneas() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    client.abrir_meta(&user, &100);
    assert!(client.try_abrir_meta(&user, &200).is_err());
}

#[test]
fn abrir_meta_falla_sin_autorizacion() {
    let (env, contract_id, user, _) = setup(false);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    assert!(client.try_abrir_meta(&user, &100).is_err());
}

#[test]
fn emergencia_devuelve_el_cien_por_ciento_del_capital() {
    let (env, contract_id, user, token_address) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token = token::Client::new(&env, &token_address);
    let saldo_inicial = token.balance(&user);

    preparar_usuario(&client, &user, 100, 4_000, 3_000, 3_000);
    client.depositar(&user, &137);
    client.retirar_todo_emergencia(&user);

    assert_eq!(token.balance(&user), saldo_inicial);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn emergencia_resetea_progreso_y_permite_abrir_otra_meta() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 100, 10_000, 0, 0);
    client.depositar(&user, &100);
    client.retirar_todo_emergencia(&user);

    assert_eq!(
        client.ver_estado(&user),
        EstadoMeta {
            abierta: false,
            meta_bps: 10_000,
            colchon_bps: 0,
            primer_paso_bps: 0,
            saldo_meta: 0,
            saldo_colchon: 0,
            saldo_primer_paso: 0,
            nivel: 0,
            insignias: 0,
            meta_base: 100,
            meta_actual: 100,
            limite_acceso_anticipado: 0,
        }
    );
    client.abrir_meta(&user, &250);
    let estado = client.ver_estado(&user);
    assert!(estado.abierta);
    assert_eq!(estado.nivel, 0);
    assert_eq!(estado.insignias, 0);
    assert_eq!(estado.saldo_meta, 0);
    assert_eq!(estado.meta_base, 250);
    assert_eq!(estado.meta_actual, 250);
    assert_eq!(estado.meta_bps, 10_000);
}

#[test]
fn acepta_configuracion_valida_y_porcentajes_cero() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    client.configurar_reparto(&user, &0, &10_000, &0);
    let estado = client.ver_estado(&user);
    assert_eq!(estado.meta_bps, 0);
    assert_eq!(estado.colchon_bps, 10_000);
    assert_eq!(estado.primer_paso_bps, 0);
}

#[test]
fn configuracion_que_no_suma_diez_mil_falla() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    assert!(client
        .try_configurar_reparto(&user, &4_000, &3_000, &2_999)
        .is_err());
}

#[test]
fn deposito_se_divide_entre_los_tres_fondos() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 1_000, 5_000, 3_000, 2_000);
    client.depositar(&user, &100);

    let estado = client.ver_estado(&user);
    assert_eq!(estado.saldo_meta, 50);
    assert_eq!(estado.saldo_colchon, 30);
    assert_eq!(estado.saldo_primer_paso, 20);
}

#[test]
fn residuo_de_redondeo_se_asigna_a_primer_paso() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 1_000, 3_333, 3_333, 3_334);
    client.depositar(&user, &2);

    let estado = client.ver_estado(&user);
    assert_eq!(estado.saldo_meta, 0);
    assert_eq!(estado.saldo_colchon, 0);
    assert_eq!(estado.saldo_primer_paso, 2);
}

#[test]
fn cambiar_porcentajes_afecta_solo_depositos_futuros() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 1_000, 5_000, 3_000, 2_000);
    client.depositar(&user, &100);
    client.configurar_reparto(&user, &0, &0, &10_000);
    client.depositar(&user, &40);

    let estado = client.ver_estado(&user);
    assert_eq!(estado.saldo_meta, 50);
    assert_eq!(estado.saldo_colchon, 30);
    assert_eq!(estado.saldo_primer_paso, 60);
}

#[test]
fn retirar_colchon_y_primer_paso_no_altera_progreso() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 10_000, 5_000, 3_000, 2_000);
    client.depositar(&user, &1_000);
    let antes = client.ver_estado(&user);
    client.retirar_colchon(&user, &100);
    client.retirar_primer_paso(&user, &50);
    let despues = client.ver_estado(&user);

    assert_eq!(despues.nivel, antes.nivel);
    assert_eq!(despues.insignias, antes.insignias);
    assert_eq!(despues.meta_actual, antes.meta_actual);
    assert_eq!(despues.saldo_meta, antes.saldo_meta);
    assert_eq!(despues.saldo_colchon, 200);
    assert_eq!(despues.saldo_primer_paso, 150);
}

#[test]
fn acceso_anticipado_se_calcula_sobre_saldo_meta_y_limite_de_nivel() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 100, 1_000, 9_000, 0);
    client.depositar(&user, &1_000);
    let estado = client.ver_estado(&user);
    assert_eq!(estado.nivel, 1);
    assert_eq!(estado.saldo_meta, 100);
    assert_eq!(estado.limite_acceso_anticipado, 10);
    assert_eq!(
        estado.limite_acceso_anticipado,
        client.acceso_desbloqueado(&user)
    );
}

#[test]
fn salida_total_devuelve_los_tres_fondos() {
    let (env, contract_id, user, token_address) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token = token::Client::new(&env, &token_address);
    let saldo_inicial = token.balance(&user);

    preparar_usuario(&client, &user, 1_000, 4_000, 3_000, 3_000);
    client.depositar(&user, &1_000);
    client.retirar_todo_emergencia(&user);

    assert_eq!(token.balance(&user), saldo_inicial);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn exceder_limite_de_acceso_revierte_el_retiro() {
    let (env, contract_id, user, token_address) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token = token::Client::new(&env, &token_address);

    preparar_usuario(&client, &user, 100, 10_000, 0, 0);
    client.depositar(&user, &100);
    let saldo_usuario = token.balance(&user);
    let saldo_contrato = token.balance(&contract_id);
    assert!(client.try_retirar_desbloqueado(&user, &11).is_err());
    assert_eq!(token.balance(&user), saldo_usuario);
    assert_eq!(token.balance(&contract_id), saldo_contrato);
}

#[test]
fn tercero_sin_autorizacion_no_puede_reconfigurar_ni_retirar() {
    let (env, contract_id, user, _) = setup(false);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    assert!(client
        .try_configurar_reparto(&user, &10_000, &0, &0)
        .is_err());
    assert!(client.try_retirar_colchon(&user, &1).is_err());
}

#[test]
fn deposito_que_supera_la_meta_sube_solo_un_nivel() {
    let (env, contract_id, user, _) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    preparar_usuario(&client, &user, 100, 10_000, 0, 0);
    client.depositar(&user, &1_000);

    let estado = client.ver_estado(&user);
    assert_eq!(estado.nivel, 1);
    assert_eq!(estado.insignias, 1);
    assert_eq!(estado.meta_actual, 170);
    assert_eq!(estado.saldo_meta, 1_000);
}

#[test]
fn saldo_del_contrato_cubre_siempre_saldos_internos() {
    let (env, contract_id, user, token_address) = setup(true);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token = token::Client::new(&env, &token_address);

    preparar_usuario(&client, &user, 100, 4_000, 3_000, 3_000);
    client.depositar(&user, &1_000);
    let estado = client.ver_estado(&user);
    let interno = estado
        .saldo_meta
        .checked_add(estado.saldo_colchon)
        .unwrap()
        .checked_add(estado.saldo_primer_paso)
        .unwrap();
    assert!(token.balance(&contract_id) >= interno);

    client.retirar_colchon(&user, &100);
    client.retirar_desbloqueado(&user, &40);
    let estado = client.ver_estado(&user);
    let interno = estado
        .saldo_meta
        .checked_add(estado.saldo_colchon)
        .unwrap()
        .checked_add(estado.saldo_primer_paso)
        .unwrap();
    assert!(token.balance(&contract_id) >= interno);
}
