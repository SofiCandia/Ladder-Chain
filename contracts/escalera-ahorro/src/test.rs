#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as AddressTestUtils, Ledger as LedgerTestUtils};
use soroban_sdk::{token, Address, Env};

fn setup(
    mock_all_auths: bool,
    primary_bps: u32,
    secondary_bps: u32,
    cooldown_ledgers: u32,
) -> (Env, Address, Address, Address) {
    setup_with_mint(
        mock_all_auths,
        primary_bps,
        secondary_bps,
        cooldown_ledgers,
        10_000_000,
    )
}

fn setup_with_mint(
    mock_all_auths: bool,
    primary_bps: u32,
    secondary_bps: u32,
    cooldown_ledgers: u32,
    mint_amount: i128,
) -> (Env, Address, Address, Address) {
    let env = Env::default();
    if mock_all_auths {
        env.mock_all_auths();
    }

    let issuer = Address::generate(&env);
    let user = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    if mock_all_auths {
        token::StellarAssetClient::new(&env, &asset).mint(&user, &mint_amount);
    }

    let contract_id = env.register(EscaleraAhorro, ());
    EscaleraAhorroClient::new(&env, &contract_id).initialize(
        &asset,
        &primary_bps,
        &secondary_bps,
        &cooldown_ledgers,
    );
    (env, contract_id, user, asset)
}

fn open_goal(client: &EscaleraAhorroClient, user: &Address, amount: i128) {
    client.abrir_meta(user, &amount);
}

#[test]
fn initialize_rechaza_porcentajes_invalidos() {
    let env = Env::default();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let contract_id = env.register(EscaleraAhorro, ());
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let asset = sac.address();

    assert!(client.try_initialize(&asset, &7_000, &3_001, &10).is_err());
    assert!(client.try_initialize(&asset, &u32::MAX, &1, &10).is_err());
}

#[test]
fn initialize_no_permite_reinicializacion() {
    let (env, contract_id, _, asset) = setup(true, 5_000, 3_000, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    assert!(client.try_initialize(&asset, &10_000, &0, &20).is_err());
}

#[test]
fn deposito_distribuye_el_monto_y_emergencia_recibe_el_residuo() {
    let (env, contract_id, user, _) = setup(true, 4_000, 3_000, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 1_000);

    client.depositar(&user, &137);
    let state = client.ver_estado(&user);

    assert_eq!(state.primary_balance, 54);
    assert_eq!(state.secondary_balance, 41);
    assert_eq!(state.emergency_balance, 42);
    assert_eq!(
        state.primary_balance + state.secondary_balance + state.emergency_balance,
        137
    );
}

#[test]
fn todo_el_residuo_de_redondeo_va_a_emergencia() {
    let (env, contract_id, user, _) = setup(true, 3_333, 3_333, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 1_000);

    client.depositar(&user, &2);
    let state = client.ver_estado(&user);

    assert_eq!(state.primary_balance, 0);
    assert_eq!(state.secondary_balance, 0);
    assert_eq!(state.emergency_balance, 2);
}

#[test]
fn rechaza_montos_cero_o_negativos() {
    let (env, contract_id, user, _) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 100);

    assert!(client.try_depositar(&user, &0).is_err());
    assert!(client.try_depositar(&user, &-1).is_err());
    assert!(client.try_retirar_meta_principal(&user, &0).is_err());
    assert!(client.try_retirar_meta_secundaria(&user, &-1).is_err());
    assert!(client.try_retirar_fondo_emergencia(&user, &0).is_err());
}

#[test]
fn meta_principal_sube_un_nivel_y_redondea_siguiente_meta_hacia_arriba() {
    let (env, contract_id, user, _) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 101);

    client.depositar(&user, &101);
    let state = client.ver_estado(&user);

    assert_eq!(state.level, 1);
    assert_eq!(state.primary_goal_amount, 142);
}

#[test]
fn un_deposito_que_supera_varias_metas_solo_avanza_un_nivel() {
    let (env, contract_id, user, _) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 100);

    client.depositar(&user, &1_000);
    let state = client.ver_estado(&user);

    assert_eq!(state.level, 1);
    assert_eq!(state.primary_goal_amount, 140);
    assert_eq!(state.primary_balance, 1_000);
}

#[test]
fn overflow_al_calcular_la_siguiente_meta_devuelve_error_tipado() {
    let (env, contract_id, user, asset) = setup_with_mint(true, 10_000, 0, 10, i128::MAX);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &asset);
    open_goal(&client, &user, i128::MAX);

    assert!(client.try_depositar(&user, &i128::MAX).is_err());
    assert_eq!(client.ver_estado(&user).primary_balance, 0);
    assert_eq!(token_client.balance(&user), i128::MAX);
}

#[test]
fn retiros_de_secundaria_y_emergencia_no_afectan_progreso_ni_cooldown() {
    let (env, contract_id, user, _) = setup(true, 5_000, 2_500, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 500);
    client.depositar(&user, &1_000);
    let before = client.ver_estado(&user);

    client.retirar_meta_secundaria(&user, &100);
    client.retirar_fondo_emergencia(&user, &100);
    let after = client.ver_estado(&user);

    assert_eq!(after.level, before.level);
    assert_eq!(after.primary_goal_amount, before.primary_goal_amount);
    assert_eq!(after.primary_balance, before.primary_balance);
    assert_eq!(after.secondary_balance, 150);
    assert_eq!(after.emergency_balance, 150);
    assert_eq!(after.badges_frozen, before.badges_frozen);
    assert_eq!(after.cooldown_until_ledger, before.cooldown_until_ledger);
}

#[test]
fn retiro_parcial_de_meta_principal_devuelve_capital_resetea_y_aplica_cooldown() {
    let (env, contract_id, user, asset) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &asset);
    let initial_user_balance = token_client.balance(&user);
    open_goal(&client, &user, 100);
    client.depositar(&user, &100);
    let ledger = env.ledger().sequence();
    env.ledger().set_sequence_number(ledger + 5);

    client.retirar_meta_principal(&user, &40);
    let state = client.ver_estado(&user);

    assert_eq!(token_client.balance(&user), initial_user_balance - 60);
    assert_eq!(token_client.balance(&contract_id), 60);
    assert_eq!(state.primary_balance, 60);
    assert_eq!(state.level, 1);
    assert!(state.badges_frozen);
    assert_eq!(state.cooldown_until_ledger, ledger + 15);
}

#[test]
fn cooldown_on_chain_impide_depositar_hasta_el_ledger_configurado() {
    let (env, contract_id, user, _) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 100);
    client.depositar(&user, &100);
    client.retirar_meta_principal(&user, &10);

    let state = client.ver_estado(&user);
    assert!(client.try_depositar(&user, &1).is_err());
    assert_eq!(client.ver_estado(&user), state);

    env.ledger()
        .set_sequence_number(state.cooldown_until_ledger);
    client.depositar(&user, &50);
    let state_after = client.ver_estado(&user);
    assert_eq!(state_after.level, 2);
    assert!(!state_after.badges_frozen);
}

#[test]
fn fondos_secundario_y_emergencia_se_pueden_retirar_totalmente() {
    let (env, contract_id, user, asset) = setup(true, 0, 5_000, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &asset);
    let initial_user_balance = token_client.balance(&user);
    open_goal(&client, &user, 100);
    client.depositar(&user, &100);

    client.retirar_meta_secundaria(&user, &50);
    client.retirar_fondo_emergencia(&user, &50);

    assert_eq!(token_client.balance(&user), initial_user_balance);
    assert_eq!(token_client.balance(&contract_id), 0);
    let state = client.ver_estado(&user);
    assert_eq!(state.level, 0);
    assert_eq!(state.primary_balance, 0);
    assert_eq!(state.secondary_balance, 0);
    assert_eq!(state.emergency_balance, 0);
}

#[test]
fn retiro_primario_superior_al_saldo_no_cambia_estado() {
    let (env, contract_id, user, _) = setup(true, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    open_goal(&client, &user, 100);
    client.depositar(&user, &100);
    let before = client.ver_estado(&user);

    assert!(client.try_retirar_meta_principal(&user, &101).is_err());
    assert_eq!(client.ver_estado(&user), before);
}

#[test]
fn operaciones_de_fondos_requieren_autorizacion_del_usuario() {
    let (env, contract_id, user, _) = setup(false, 10_000, 0, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);

    assert!(client.try_abrir_meta(&user, &100).is_err());
    assert!(client.try_depositar(&user, &1).is_err());
    assert!(client.try_retirar_meta_principal(&user, &1).is_err());
    assert!(client.try_retirar_meta_secundaria(&user, &1).is_err());
    assert!(client.try_retirar_fondo_emergencia(&user, &1).is_err());
}

#[test]
fn contrato_cubre_los_saldos_persistidos_tras_retiros() {
    let (env, contract_id, user, asset) = setup(true, 5_000, 2_500, 10);
    let client = EscaleraAhorroClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &asset);
    open_goal(&client, &user, 10_000);
    client.depositar(&user, &1_000);
    client.retirar_meta_secundaria(&user, &50);
    client.retirar_fondo_emergencia(&user, &50);
    let state = client.ver_estado(&user);

    assert_eq!(
        token_client.balance(&contract_id),
        state.primary_balance + state.secondary_balance + state.emergency_balance
    );
}
