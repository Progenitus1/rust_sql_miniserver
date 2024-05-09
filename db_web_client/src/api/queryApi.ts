import { IDBResponse } from "../types";
import { axiosInstance } from "./base";

export async function dbQuery(query: string): Promise<IDBResponse> {
    const response = await axiosInstance.post<IDBResponse>('/query', { query });
    return response.data;
}