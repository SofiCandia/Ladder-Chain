#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, token,
    Address, Env, Symbol,
};

const INCREMENTO_META_PCT: i128 = 170;
const PCT_ACCESO_POR_NIVEL: i128 = 10;
const PCT_ACCESO_MAX: i128 = 50;
const PUNTOS_BASE: u32 = 10_000;
const TTL_MINIMO: u32 = 100_000;
const TTL_EXTENDIDO: u32 = 200_000;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Overflow = 1,
    YaInicializado = 2,
    NoInicializado = 3,
    MetaYaAbierta = 4,
    MetaInvalida = 5,
    MontoInvalido = 6,
    MetaCerrada = 7,
    ExcedeAccesoDesbloqueado = 8,
    SinSaldoParaRetirar = 9,
    SaldoInsuficiente = 10,
    RepartoInvalido = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstadoMeta {
    pub abierta: bool,
    pub meta_bps: u32,
    pub colchon_bps: u32,
    pub primer_paso_bps: u32,
    pub saldo_meta: i128,
    pub saldo_colchon: i128,
    pub saldo_primer_paso: i128,
    pub nivel: u32,
    pub insignias: u32,
    pub meta_base: i128,
    pub meta_actual: i128,
    pub limite_acceso_anticipado: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventoConfiguracion {
    #[topic]
    pub user: Address,
    pub meta_bps: u32,
    pub colchon_bps: u32,
    pub primer_paso_bps: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventoDeposito {
    #[topic]
    pub user: Address,
    pub monto: i128,
    pub meta: i128,
    pub colchon: i128,
    pub primer_paso: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventoNivel {
    #[topic]
    pub user: Address,
    pub nivel: u32,
    pub insignias: u32,
    pub meta_actual: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventoRetiro {
    #[topic]
    pub user: Address,
    pub fondo: Symbol,
    pub monto: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventoSalidaTotal {
    #[topic]
    pub user: Address,
    pub total: i128,
    pub saldo_meta: i128,
    pub saldo_colchon: i128,
    pub saldo_primer_paso: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct EstadoUsuario {
    abierta: bool,
    meta_bps: u32,
    colchon_bps: u32,
    primer_paso_bps: u32,
    saldo_meta: i128,
    saldo_colchon: i128,
    saldo_primer_paso: i128,
    nivel: u32,
    insignias: u32,
    meta_base: i128,
    meta_actual: i128,
}

impl EstadoUsuario {
    fn inicial() -> Self {
        Self {
            abierta: false,
            meta_bps: 0,
            colchon_bps: 0,
            primer_paso_bps: 0,
            saldo_meta: 0,
            saldo_colchon: 0,
            saldo_primer_paso: 0,
            nivel: 0,
            insignias: 0,
            meta_base: 0,
            meta_actual: 0,
        }
    }
}

#[contracttype]
#[derive(Clone)]
enum Clave {
    Token,
    Usuario(Address),
}

#[contract]
pub struct EscaleraAhorro;

#[contractimpl]
impl EscaleraAhorro {
    pub fn initialize(env: Env, token: Address) -> Result<(), Error> {
        let storage = env.storage().instance();
        if storage.has(&Clave::Token) {
            return Err(Error::YaInicializado);
        }
        storage.set(&Clave::Token, &token);
        storage.extend_ttl(TTL_MINIMO, TTL_EXTENDIDO);
        Ok(())
    }

    pub fn abrir_meta(env: Env, user: Address, meta_inicial: i128) -> Result<(), Error> {
        user.require_auth();
        if meta_inicial <= 0 {
            return Err(Error::MetaInvalida);
        }

        let mut estado = Self::leer_estado(&env, &user);
        if estado.abierta {
            return Err(Error::MetaYaAbierta);
        }
        estado.abierta = true;
        estado.nivel = 0;
        estado.insignias = 0;
        estado.saldo_meta = 0;
        estado.saldo_colchon = 0;
        estado.saldo_primer_paso = 0;
        estado.meta_base = meta_inicial;
        estado.meta_actual = meta_inicial;
        Self::guardar_estado(&env, &user, &estado);
        Ok(())
    }

    pub fn configurar_reparto(
        env: Env,
        user: Address,
        meta_bps: u32,
        colchon_bps: u32,
        primer_paso_bps: u32,
    ) -> Result<(), Error> {
        user.require_auth();
        let suma = meta_bps
            .checked_add(colchon_bps)
            .and_then(|valor| valor.checked_add(primer_paso_bps))
            .ok_or(Error::Overflow)?;
        if suma != PUNTOS_BASE {
            return Err(Error::RepartoInvalido);
        }

        let mut estado = Self::leer_estado(&env, &user);
        estado.meta_bps = meta_bps;
        estado.colchon_bps = colchon_bps;
        estado.primer_paso_bps = primer_paso_bps;
        Self::guardar_estado(&env, &user, &estado);
        EventoConfiguracion {
            user,
            meta_bps,
            colchon_bps,
            primer_paso_bps,
        }
        .publish(&env);
        Ok(())
    }

    pub fn depositar(env: Env, user: Address, monto: i128) -> Result<(), Error> {
        user.require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        let mut estado = Self::leer_estado_abierta(&env, &user)?;
        Self::validar_reparto(&estado)?;

        let meta = Self::repartir(monto, estado.meta_bps)?;
        let colchon = Self::repartir(monto, estado.colchon_bps)?;
        let primer_paso = monto
            .checked_sub(meta)
            .ok_or(Error::Overflow)?
            .checked_sub(colchon)
            .ok_or(Error::Overflow)?;

        estado.saldo_meta = estado.saldo_meta.checked_add(meta).ok_or(Error::Overflow)?;
        estado.saldo_colchon = estado
            .saldo_colchon
            .checked_add(colchon)
            .ok_or(Error::Overflow)?;
        estado.saldo_primer_paso = estado
            .saldo_primer_paso
            .checked_add(primer_paso)
            .ok_or(Error::Overflow)?;

        let subio_nivel = estado.saldo_meta >= estado.meta_actual;
        if subio_nivel {
            estado.nivel = estado.nivel.checked_add(1).ok_or(Error::Overflow)?;
            estado.insignias = estado.insignias.checked_add(1).ok_or(Error::Overflow)?;
            estado.meta_actual = estado
                .meta_actual
                .checked_mul(INCREMENTO_META_PCT)
                .ok_or(Error::Overflow)?
                .checked_div(100)
                .ok_or(Error::Overflow)?;
        }

        let token = Self::leer_token(&env)?;
        token::Client::new(&env, &token).transfer(&user, &env.current_contract_address(), &monto);
        Self::guardar_estado(&env, &user, &estado);

        EventoDeposito {
            user: user.clone(),
            monto,
            meta,
            colchon,
            primer_paso,
        }
        .publish(&env);
        if subio_nivel {
            EventoNivel {
                user,
                nivel: estado.nivel,
                insignias: estado.insignias,
                meta_actual: estado.meta_actual,
            }
            .publish(&env);
        }
        Ok(())
    }

    pub fn acceso_desbloqueado(env: Env, user: Address) -> Result<i128, Error> {
        let estado = Self::leer_estado(&env, &user);
        Self::calcular_acceso(estado.saldo_meta, estado.nivel)
    }

    pub fn retirar_desbloqueado(env: Env, user: Address, monto: i128) -> Result<(), Error> {
        user.require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        let mut estado = Self::leer_estado_abierta(&env, &user)?;
        let desbloqueado = Self::calcular_acceso(estado.saldo_meta, estado.nivel)?;
        if monto > desbloqueado {
            return Err(Error::ExcedeAccesoDesbloqueado);
        }
        estado.saldo_meta = estado
            .saldo_meta
            .checked_sub(monto)
            .ok_or(Error::Overflow)?;
        Self::transferir_y_guardar(&env, &user, &estado, monto)?;
        Self::emitir_retiro(&env, &user, symbol_short!("meta"), monto);
        Ok(())
    }

    pub fn retirar_colchon(env: Env, user: Address, monto: i128) -> Result<(), Error> {
        Self::retirar_fondo(env, user, monto, symbol_short!("colchon"))
    }

    pub fn retirar_primer_paso(env: Env, user: Address, monto: i128) -> Result<(), Error> {
        Self::retirar_fondo(env, user, monto, symbol_short!("primer"))
    }

    pub fn retirar_todo_emergencia(env: Env, user: Address) -> Result<(), Error> {
        user.require_auth();
        let mut estado = Self::leer_estado_abierta(&env, &user)?;
        let total = estado
            .saldo_meta
            .checked_add(estado.saldo_colchon)
            .ok_or(Error::Overflow)?
            .checked_add(estado.saldo_primer_paso)
            .ok_or(Error::Overflow)?;
        if total <= 0 {
            return Err(Error::SinSaldoParaRetirar);
        }

        let saldos = (
            estado.saldo_meta,
            estado.saldo_colchon,
            estado.saldo_primer_paso,
        );
        estado.abierta = false;
        estado.saldo_meta = 0;
        estado.saldo_colchon = 0;
        estado.saldo_primer_paso = 0;
        estado.nivel = 0;
        estado.insignias = 0;
        estado.meta_actual = estado.meta_base;

        let token = Self::leer_token(&env)?;
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &user, &total);
        Self::guardar_estado(&env, &user, &estado);
        EventoSalidaTotal {
            user,
            total,
            saldo_meta: saldos.0,
            saldo_colchon: saldos.1,
            saldo_primer_paso: saldos.2,
        }
        .publish(&env);
        Ok(())
    }

    pub fn ver_estado(env: Env, user: Address) -> Result<EstadoMeta, Error> {
        let estado = Self::leer_estado(&env, &user);
        let limite_acceso_anticipado = Self::calcular_acceso(estado.saldo_meta, estado.nivel)?;
        Ok(EstadoMeta {
            abierta: estado.abierta,
            meta_bps: estado.meta_bps,
            colchon_bps: estado.colchon_bps,
            primer_paso_bps: estado.primer_paso_bps,
            saldo_meta: estado.saldo_meta,
            saldo_colchon: estado.saldo_colchon,
            saldo_primer_paso: estado.saldo_primer_paso,
            nivel: estado.nivel,
            insignias: estado.insignias,
            meta_base: estado.meta_base,
            meta_actual: estado.meta_actual,
            limite_acceso_anticipado,
        })
    }
}

impl EscaleraAhorro {
    fn leer_token(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&Clave::Token)
            .ok_or(Error::NoInicializado)
    }

    fn leer_estado(env: &Env, user: &Address) -> EstadoUsuario {
        let clave = Clave::Usuario(user.clone());
        match env.storage().persistent().get(&clave) {
            Some(estado) => estado,
            None => EstadoUsuario::inicial(),
        }
    }

    fn leer_estado_abierta(env: &Env, user: &Address) -> Result<EstadoUsuario, Error> {
        let estado = Self::leer_estado(env, user);
        if estado.abierta {
            Ok(estado)
        } else {
            Err(Error::MetaCerrada)
        }
    }

    fn validar_reparto(estado: &EstadoUsuario) -> Result<(), Error> {
        let suma = estado
            .meta_bps
            .checked_add(estado.colchon_bps)
            .and_then(|valor| valor.checked_add(estado.primer_paso_bps))
            .ok_or(Error::Overflow)?;
        if suma == PUNTOS_BASE {
            Ok(())
        } else {
            Err(Error::RepartoInvalido)
        }
    }

    fn repartir(monto: i128, bps: u32) -> Result<i128, Error> {
        monto
            .checked_mul(i128::from(bps))
            .ok_or(Error::Overflow)?
            .checked_div(i128::from(PUNTOS_BASE))
            .ok_or(Error::Overflow)
    }

    fn calcular_acceso(saldo_meta: i128, nivel: u32) -> Result<i128, Error> {
        let porcentaje_nivel = i128::from(nivel)
            .checked_mul(PCT_ACCESO_POR_NIVEL)
            .ok_or(Error::Overflow)?;
        let porcentaje = if porcentaje_nivel < PCT_ACCESO_MAX {
            porcentaje_nivel
        } else {
            PCT_ACCESO_MAX
        };
        let centenas = saldo_meta.checked_div(100).ok_or(Error::Overflow)?;
        let residuo = saldo_meta.checked_rem(100).ok_or(Error::Overflow)?;
        let parte_entera = centenas.checked_mul(porcentaje).ok_or(Error::Overflow)?;
        let parte_residual = residuo
            .checked_mul(porcentaje)
            .ok_or(Error::Overflow)?
            .checked_div(100)
            .ok_or(Error::Overflow)?;
        parte_entera
            .checked_add(parte_residual)
            .ok_or(Error::Overflow)
    }

    fn transferir_y_guardar(
        env: &Env,
        user: &Address,
        estado: &EstadoUsuario,
        monto: i128,
    ) -> Result<(), Error> {
        let token = Self::leer_token(env)?;
        token::Client::new(env, &token).transfer(&env.current_contract_address(), user, &monto);
        Self::guardar_estado(env, user, estado);
        Ok(())
    }

    fn retirar_fondo(env: Env, user: Address, monto: i128, fondo: Symbol) -> Result<(), Error> {
        user.require_auth();
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        let mut estado = Self::leer_estado_abierta(&env, &user)?;
        let saldo = if fondo == symbol_short!("colchon") {
            &mut estado.saldo_colchon
        } else {
            &mut estado.saldo_primer_paso
        };
        if monto > *saldo {
            return Err(Error::SaldoInsuficiente);
        }
        *saldo = saldo.checked_sub(monto).ok_or(Error::Overflow)?;
        Self::transferir_y_guardar(&env, &user, &estado, monto)?;
        Self::emitir_retiro(&env, &user, fondo, monto);
        Ok(())
    }

    fn emitir_retiro(env: &Env, user: &Address, fondo: Symbol, monto: i128) {
        EventoRetiro {
            user: user.clone(),
            fondo,
            monto,
        }
        .publish(env);
    }

    fn guardar_estado(env: &Env, user: &Address, estado: &EstadoUsuario) {
        let storage = env.storage().persistent();
        let clave = Clave::Usuario(user.clone());
        storage.set(&clave, estado);
        storage.extend_ttl(&clave, TTL_MINIMO, TTL_EXTENDIDO);
    }
}

mod test;
