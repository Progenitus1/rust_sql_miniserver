import { createColumnHelper, useReactTable, getCoreRowModel, ColumnDef } from "@tanstack/react-table";
import { FC } from "react";
import { IDBDataValue, IDBRow, IDBTable } from "../types";

interface IProps {
    dbTable: IDBTable;
}

export const DBTable: FC<IProps> = ({ dbTable }) => {
    return (
        <div className="db-table">
            <table>
                <thead>
                    <tr>
                        {dbTable.columns.map((column) => (
                            <th key={column.name}>{column.name}</th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {dbTable.rows.map((row, rowIndex) => (
                        <tr key={rowIndex}>
                            {dbTable.columns.map((column, colIndex) => (
                                <td key={column.name}>
                                    {row.values[colIndex][column.dataType] || (
                                        <span className="text-light-gray/30">null</span>
                                    )}
                                </td>
                            ))}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
};
