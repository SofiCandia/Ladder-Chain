import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const Errors = {
  1: {message:"Overflow"},
  2: {message:"YaInicializado"},
  3: {message:"NoInicializado"},
  4: {message:"RepartoInvalido"},
  5: {message:"MetaInvalida"},
  6: {message:"MetaYaAbierta"},
  7: {message:"MontoInvalido"},
  8: {message:"MetaCerrada"},
  9: {message:"SaldoInsuficiente"},
  10: {message:"SinSaldoParaRetirar"},
  11: {message:"CooldownActivo"}
}


export interface EstadoMeta {
  badges_frozen: boolean;
  cooldown_until_ledger: u32;
  emergency_balance: i128;
  level: u32;
  primary_balance: i128;
  primary_goal_amount: i128;
  secondary_balance: i128;
}





export interface Client {
  /**
   * Construct and simulate a depositar transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  depositar: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a abrir_meta transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  abrir_meta: ({user, primary_goal_amount}: {user: string, primary_goal_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({asset, primary_bps, secondary_bps, cooldown_ledgers}: {asset: string, primary_bps: u32, secondary_bps: u32, cooldown_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a ver_estado transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  ver_estado: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<EstadoMeta>>>

  /**
   * Construct and simulate a retirar_meta_principal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_meta_principal: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a retirar_meta_secundaria transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_meta_secundaria: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a retirar_fondo_emergencia transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_fondo_emergencia: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAACwAAAAAAAAAIT3ZlcmZsb3cAAAABAAAAAAAAAA5ZYUluaWNpYWxpemFkbwAAAAAAAgAAAAAAAAAOTm9JbmljaWFsaXphZG8AAAAAAAMAAAAAAAAAD1JlcGFydG9JbnZhbGlkbwAAAAAEAAAAAAAAAAxNZXRhSW52YWxpZGEAAAAFAAAAAAAAAA1NZXRhWWFBYmllcnRhAAAAAAAABgAAAAAAAAANTW9udG9JbnZhbGlkbwAAAAAAAAcAAAAAAAAAC01ldGFDZXJyYWRhAAAAAAgAAAAAAAAAEVNhbGRvSW5zdWZpY2llbnRlAAAAAAAACQAAAAAAAAATU2luU2FsZG9QYXJhUmV0aXJhcgAAAAAKAAAAAAAAAA5Db29sZG93bkFjdGl2bwAAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAACkVzdGFkb01ldGEAAAAAAAcAAAAAAAAADWJhZGdlc19mcm96ZW4AAAAAAAABAAAAAAAAABVjb29sZG93bl91bnRpbF9sZWRnZXIAAAAAAAAEAAAAAAAAABFlbWVyZ2VuY3lfYmFsYW5jZQAAAAAAAAsAAAAAAAAABWxldmVsAAAAAAAABAAAAAAAAAAPcHJpbWFyeV9iYWxhbmNlAAAAAAsAAAAAAAAAE3ByaW1hcnlfZ29hbF9hbW91bnQAAAAACwAAAAAAAAARc2Vjb25kYXJ5X2JhbGFuY2UAAAAAAAAL",
        "AAAABQAAAAAAAAAAAAAADERlcG9zaXRFdmVudAAAAAEAAAAHZGVwb3NpdAAAAAAFAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAAAAAAB3ByaW1hcnkAAAAACwAAAAAAAAAAAAAACXNlY29uZGFyeQAAAAAAAAsAAAAAAAAAAAAAAAllbWVyZ2VuY3kAAAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADExldmVsVXBFdmVudAAAAAEAAAAIbGV2ZWxfdXAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAAAAAADnByZXZpb3VzX2xldmVsAAAAAAAEAAAAAAAAAAAAAAAJbmV3X2xldmVsAAAAAAAABAAAAAAAAAAAAAAACW5leHRfZ29hbAAAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGEVtZXJnZW5jeVdpdGhkcmF3YWxFdmVudAAAAAEAAAAUZW1lcmdlbmN5X3dpdGhkcmF3YWwAAAACAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAG1ByaW1hcnlXaXRoZHJhd2FsUmVzZXRFdmVudAAAAAABAAAAGHByaW1hcnlfd2l0aGRyYXdhbF9yZXNldAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAAAAAAVY29vbGRvd25fdW50aWxfbGVkZ2VyAAAAAAAABAAAAAAAAAAC",
        "AAAAAAAAAAAAAAAJZGVwb3NpdGFyAAAAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAKYWJyaXJfbWV0YQAAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAAE3ByaW1hcnlfZ29hbF9hbW91bnQAAAAACwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAABAAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAtwcmltYXJ5X2JwcwAAAAAEAAAAAAAAAA1zZWNvbmRhcnlfYnBzAAAAAAAABAAAAAAAAAAQY29vbGRvd25fbGVkZ2VycwAAAAQAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAKdmVyX2VzdGFkbwAAAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAB9AAAAAKRXN0YWRvTWV0YQAAAAAAAw==",
        "AAAAAAAAAAAAAAAWcmV0aXJhcl9tZXRhX3ByaW5jaXBhbAAAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAXcmV0aXJhcl9tZXRhX3NlY3VuZGFyaWEAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAYcmV0aXJhcl9mb25kb19lbWVyZ2VuY2lhAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAAAgAAAAM=" ]),
      options
    )
  }
  public readonly fromJSON = {
    depositar: this.txFromJSON<Result<void>>,
        abrir_meta: this.txFromJSON<Result<void>>,
        initialize: this.txFromJSON<Result<void>>,
        ver_estado: this.txFromJSON<Result<EstadoMeta>>,
        retirar_meta_principal: this.txFromJSON<Result<void>>,
        retirar_meta_secundaria: this.txFromJSON<Result<void>>,
        retirar_fondo_emergencia: this.txFromJSON<Result<void>>
  }
}