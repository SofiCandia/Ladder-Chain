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
  4: {message:"MetaYaAbierta"},
  5: {message:"MetaInvalida"},
  6: {message:"MontoInvalido"},
  7: {message:"MetaCerrada"},
  8: {message:"ExcedeAccesoDesbloqueado"},
  9: {message:"SinSaldoParaRetirar"},
  10: {message:"SaldoInsuficiente"},
  11: {message:"RepartoInvalido"}
}


export interface EstadoMeta {
  abierta: boolean;
  colchon_bps: u32;
  insignias: u32;
  limite_acceso_anticipado: i128;
  meta_actual: i128;
  meta_base: i128;
  meta_bps: u32;
  nivel: u32;
  primer_paso_bps: u32;
  saldo_colchon: i128;
  saldo_meta: i128;
  saldo_primer_paso: i128;
}






export interface Client {
  /**
   * Construct and simulate a depositar transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  depositar: ({user, monto}: {user: string, monto: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a abrir_meta transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  abrir_meta: ({user, meta_inicial}: {user: string, meta_inicial: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({token}: {token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a ver_estado transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  ver_estado: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<EstadoMeta>>>

  /**
   * Construct and simulate a retirar_colchon transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_colchon: ({user, monto}: {user: string, monto: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a configurar_reparto transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  configurar_reparto: ({user, meta_bps, colchon_bps, primer_paso_bps}: {user: string, meta_bps: u32, colchon_bps: u32, primer_paso_bps: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a acceso_desbloqueado transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  acceso_desbloqueado: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a retirar_primer_paso transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_primer_paso: ({user, monto}: {user: string, monto: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a retirar_desbloqueado transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_desbloqueado: ({user, monto}: {user: string, monto: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a retirar_todo_emergencia transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  retirar_todo_emergencia: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
      new ContractSpec([ "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",
        "AAAAAAAAAAAAAAAPcmV0aXJhcl9jb2xjaG9uAAAAAAIAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVtb250bwAAAAAAAAsAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAASY29uZmlndXJhcl9yZXBhcnRvAAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAIbWV0YV9icHMAAAAEAAAAAAAAAAtjb2xjaG9uX2JwcwAAAAAEAAAAAAAAAA9wcmltZXJfcGFzb19icHMAAAAABAAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAATYWNjZXNvX2Rlc2Jsb3F1ZWFkbwAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAACwAAAAM=",
        "AAAAAAAAAAAAAAATcmV0aXJhcl9wcmltZXJfcGFzbwAAAAACAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFbW9udG8AAAAAAAALAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAUcmV0aXJhcl9kZXNibG9xdWVhZG8AAAACAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFbW9udG8AAAAAAAALAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAXcmV0aXJhcl90b2RvX2VtZXJnZW5jaWEAAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAAAIAAAAD" ]),
      options
    )
  }
  public readonly fromJSON = {
    depositar: this.txFromJSON<Result<void>>,
        abrir_meta: this.txFromJSON<Result<void>>,
        initialize: this.txFromJSON<Result<void>>,
        ver_estado: this.txFromJSON<Result<EstadoMeta>>,
        retirar_colchon: this.txFromJSON<Result<void>>,
        configurar_reparto: this.txFromJSON<Result<void>>,
        acceso_desbloqueado: this.txFromJSON<Result<i128>>,
        retirar_primer_paso: this.txFromJSON<Result<void>>,
        retirar_desbloqueado: this.txFromJSON<Result<void>>,
        retirar_todo_emergencia: this.txFromJSON<Result<void>>
  }
}