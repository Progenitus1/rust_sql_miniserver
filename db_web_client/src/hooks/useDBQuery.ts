import { useMutation } from "@tanstack/react-query";
import { dbQuery } from "../api";

export function useDBQuery() {
    return useMutation({
        mutationKey: ['dbQuery'],
        mutationFn: async (query: string) => dbQuery(query)
    });
}