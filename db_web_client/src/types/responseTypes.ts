import { IDBTable } from "./dbTypes";

export enum RespStatus {
    Error = 'err',
    Ok = 'ok',
}

export interface IDBResponse {
    status: RespStatus;
    message?: string;
    data?: IDBTable;
    duration: string;
}
