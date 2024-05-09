export enum DBDataType {
    Int = 'INT',
    String = 'STRING',
    Null = 'NULL',
    Float = 'FLOAT'
}

export interface IDBColumn {
    name: string;
    dataType: DBDataType
}

export interface IDBDataValue {
    [DBDataType.Int]?: number;
    [DBDataType.String]?: string;
    [DBDataType.Null]?: null;
    [DBDataType.Float]?: number;
}

export interface IDBRow {
    values: IDBDataValue[]
}

export interface IDBTable {
    columns: IDBColumn[]
    rows: IDBRow[]
}
