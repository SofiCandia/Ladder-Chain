#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
};

const BPS_TOTAL: u32 = 10_000;
const TTL_MINIMUM: u32 = 100_000;
const TTL_RENEWAL: u32 = 200_000;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Overflow = 1,
    YaInicializado = 2,
    NoInicializado = 3,
    RepartoInvalido = 4,
    MetaInvalida = 5,
    MetaYaAbierta = 6,
    MontoInvalido = 7,
    MetaCerrada = 8,
    SaldoInsuficiente = 9,
    SinSaldoParaRetirar = 10,
    CooldownActivo = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstadoMeta {
    pub primary_balance: i128,
    pub secondary_balance: i128,
    pub emergency_balance: i128,
    pub primary_goal_amount: i128,
    pub level: u32,
    pub badges_frozen: bool,
    pub cooldown_until_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct Config {
    asset: Address,
    primary_bps: u32,
    secondary_bps: u32,
    emergency_bps: u32,
    cooldown_ledgers: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct UserState {
    primary_balance: i128,
    secondary_balance: i128,
    emergency_balance: i128,
    primary_goal_amount: i128,
    level: u32,
    badges_frozen: bool,
    cooldown_until_ledger: u32,
}

impl UserState {
    fn empty() -> Self {
        Self {
            primary_balance: 0,
            secondary_balance: 0,
            emergency_balance: 0,
            primary_goal_amount: 0,
            level: 0,
            badges_frozen: false,
            cooldown_until_ledger: 0,
        }
    }
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Config,
    UserState(Address),
}

#[contractevent(topics = ["deposit"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    #[topic]
    pub user: Address,
    pub amount: i128,
    pub primary: i128,
    pub secondary: i128,
    pub emergency: i128,
}

#[contractevent(topics = ["level_up"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LevelUpEvent {
    #[topic]
    pub user: Address,
    pub previous_level: u32,
    pub new_level: u32,
    pub next_goal: i128,
}

#[contractevent(topics = ["primary_withdrawal_reset"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimaryWithdrawalResetEvent {
    #[topic]
    pub user: Address,
    pub amount: i128,
    pub cooldown_until_ledger: u32,
}

#[contractevent(topics = ["emergency_withdrawal"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyWithdrawalEvent {
    #[topic]
    pub user: Address,
    pub amount: i128,
}

#[contract]
pub struct EscaleraAhorro;

#[contractimpl]
impl EscaleraAhorro {
    pub fn initialize(
        env: Env,
        asset: Address,
        primary_bps: u32,
        secondary_bps: u32,
        cooldown_ledgers: u32,
    ) -> Result<(), Error> {
        let storage = env.storage().instance();
        let key = DataKey::Config;
        if storage.has(&key) {
            return Err(Error::YaInicializado);
        }
        let assigned_bps = primary_bps
            .checked_add(secondary_bps)
            .ok_or(Error::RepartoInvalido)?;
        if assigned_bps > BPS_TOTAL {
            return Err(Error::RepartoInvalido);
        }
        let emergency_bps = BPS_TOTAL
            .checked_sub(assigned_bps)
            .ok_or(Error::RepartoInvalido)?;
        storage.set(
            &key,
            &Config {
                asset,
                primary_bps,
                secondary_bps,
                emergency_bps,
                cooldown_ledgers,
            },
        );
        storage.extend_ttl(TTL_MINIMUM, TTL_RENEWAL);
        Ok(())
    }

    pub fn abrir_meta(env: Env, user: Address, primary_goal_amount: i128) -> Result<(), Error> {
        user.require_auth();
        if primary_goal_amount <= 0 {
            return Err(Error::MetaInvalida);
        }
        Self::config(&env)?;
        let mut state = Self::read_user_state(&env, &user);
        if state.primary_goal_amount != 0 {
            return Err(Error::MetaYaAbierta);
        }
        state.primary_goal_amount = primary_goal_amount;
        Self::save_user_state(&env, &user, &state);
        Ok(())
    }

    pub fn depositar(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::MontoInvalido);
        }
        let config = Self::config(&env)?;
        Self::extend_config_ttl(&env);
        let mut state = Self::read_user_state(&env, &user);
        if state.primary_goal_amount <= 0 {
            return Err(Error::MetaCerrada);
        }
        let ledger = env.ledger().sequence();
        if ledger < state.cooldown_until_ledger {
            return Err(Error::CooldownActivo);
        }

        let primary = Self::allocation(amount, config.primary_bps)?;
        let secondary = Self::allocation(amount, config.secondary_bps)?;
        let emergency = amount
            .checked_sub(primary)
            .and_then(|remainder| remainder.checked_sub(secondary))
            .ok_or(Error::Overflow)?;
        state.primary_balance = state
            .primary_balance
            .checked_add(primary)
            .ok_or(Error::Overflow)?;
        state.secondary_balance = state
            .secondary_balance
            .checked_add(secondary)
            .ok_or(Error::Overflow)?;
        state.emergency_balance = state
            .emergency_balance
            .checked_add(emergency)
            .ok_or(Error::Overflow)?;

        if state.badges_frozen {
            state.badges_frozen = false;
        }
        let previous_level = state.level;
        let mut next_goal = state.primary_goal_amount;
        if state.primary_balance >= state.primary_goal_amount {
            state.level = state.level.checked_add(1).ok_or(Error::Overflow)?;
            next_goal = state
                .primary_goal_amount
                .checked_mul(140)
                .ok_or(Error::Overflow)?
                .checked_add(99)
                .ok_or(Error::Overflow)?
                .checked_div(100)
                .ok_or(Error::Overflow)?;
            state.primary_goal_amount = next_goal;
        }

        token::Client::new(&env, &config.asset).transfer(
            &user,
            &env.current_contract_address(),
            &amount,
        );
        Self::save_user_state(&env, &user, &state);
        DepositEvent {
            user: user.clone(),
            amount,
            primary,
            secondary,
            emergency,
        }
        .publish(&env);
        if state.level != previous_level {
            LevelUpEvent {
                user,
                previous_level,
                new_level: state.level,
                next_goal,
            }
            .publish(&env);
        }
        Ok(())
    }

    pub fn retirar_meta_principal(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::MontoInvalido);
        }
        let config = Self::config(&env)?;
        Self::extend_config_ttl(&env);
        let mut state = Self::read_user_state(&env, &user);
        if amount > state.primary_balance {
            return Err(Error::SaldoInsuficiente);
        }
        state.primary_balance = state
            .primary_balance
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        state.level = 1;
        state.badges_frozen = true;
        state.cooldown_until_ledger = env
            .ledger()
            .sequence()
            .checked_add(config.cooldown_ledgers)
            .ok_or(Error::Overflow)?;
        Self::transfer_to_user(&env, &config.asset, &user, amount);
        Self::save_user_state(&env, &user, &state);
        PrimaryWithdrawalResetEvent {
            user,
            amount,
            cooldown_until_ledger: state.cooldown_until_ledger,
        }
        .publish(&env);
        Ok(())
    }

    pub fn retirar_meta_secundaria(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::MontoInvalido);
        }
        let config = Self::config(&env)?;
        Self::extend_config_ttl(&env);
        let mut state = Self::read_user_state(&env, &user);
        if amount > state.secondary_balance {
            return Err(Error::SaldoInsuficiente);
        }
        state.secondary_balance = state
            .secondary_balance
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        Self::transfer_to_user(&env, &config.asset, &user, amount);
        Self::save_user_state(&env, &user, &state);
        Ok(())
    }

    pub fn retirar_fondo_emergencia(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::MontoInvalido);
        }
        let config = Self::config(&env)?;
        Self::extend_config_ttl(&env);
        let mut state = Self::read_user_state(&env, &user);
        if amount > state.emergency_balance {
            return Err(Error::SaldoInsuficiente);
        }
        state.emergency_balance = state
            .emergency_balance
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        Self::transfer_to_user(&env, &config.asset, &user, amount);
        Self::save_user_state(&env, &user, &state);
        EmergencyWithdrawalEvent { user, amount }.publish(&env);
        Ok(())
    }

    pub fn ver_estado(env: Env, user: Address) -> Result<EstadoMeta, Error> {
        let _config = Self::config(&env)?;
        let state = Self::read_user_state(&env, &user);
        Ok(EstadoMeta {
            primary_balance: state.primary_balance,
            secondary_balance: state.secondary_balance,
            emergency_balance: state.emergency_balance,
            primary_goal_amount: state.primary_goal_amount,
            level: state.level,
            badges_frozen: state.badges_frozen,
            cooldown_until_ledger: state.cooldown_until_ledger,
        })
    }
}

impl EscaleraAhorro {
    fn config(env: &Env) -> Result<Config, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NoInicializado)
    }

    fn extend_config_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_MINIMUM, TTL_RENEWAL);
    }

    fn read_user_state(env: &Env, user: &Address) -> UserState {
        match env
            .storage()
            .persistent()
            .get(&DataKey::UserState(user.clone()))
        {
            Some(state) => state,
            None => UserState::empty(),
        }
    }

    fn save_user_state(env: &Env, user: &Address, state: &UserState) {
        let storage = env.storage().persistent();
        let key = DataKey::UserState(user.clone());
        storage.set(&key, state);
        storage.extend_ttl(&key, TTL_MINIMUM, TTL_RENEWAL);
    }

    fn allocation(amount: i128, bps: u32) -> Result<i128, Error> {
        amount
            .checked_mul(i128::from(bps))
            .ok_or(Error::Overflow)?
            .checked_div(i128::from(BPS_TOTAL))
            .ok_or(Error::Overflow)
    }

    fn transfer_to_user(env: &Env, asset: &Address, user: &Address, amount: i128) {
        token::Client::new(env, asset).transfer(&env.current_contract_address(), user, &amount);
    }
}

mod test;
