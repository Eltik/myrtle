import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { localesQueryOptions } from "#/lib/api/admin";

export interface IAdminLocale {
    code: string;
    englishName: string;
    nativeName: string;
    isSource: boolean;
}

/**
 * The locale table, for language names. Pass the role check: any panel role
 * may read it, but only the roles whose screen shows a language should fetch.
 * `name` falls back to the code for a locale the table does not list.
 */
export function useAdminLocales(enabled: boolean): { list: IAdminLocale[]; name: (code: string) => string; loaded: boolean } {
    const { isAuthenticated } = useAuth();
    const query = useQuery({ ...localesQueryOptions(isAuthenticated), enabled: enabled && isAuthenticated });
    return useMemo(() => {
        const list = (query.data ?? []).map((l) => ({ code: l.code, englishName: l.english_name, nativeName: l.native_name, isSource: l.is_source }));
        return { list, name: (code: string) => list.find((l) => l.code === code)?.englishName ?? code, loaded: query.data !== undefined };
    }, [query.data]);
}
