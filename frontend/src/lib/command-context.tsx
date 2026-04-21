import { createContext, useContext, useState } from "react";
import { SearchCommand } from "#/components/ui/search-command";

interface ICommandContextValue {
    open: () => void;
}

const CommandContext = createContext<ICommandContextValue>({ open: () => {} });

export function useCommand() {
    return useContext(CommandContext);
}

export function CommandProvider({ children }: { children: React.ReactNode }) {
    const [cmdOpen, setCmdOpen] = useState(false);

    return (
        <CommandContext.Provider value={{ open: () => setCmdOpen(true) }}>
            <SearchCommand open={cmdOpen} onOpenChange={setCmdOpen} />
            {children}
        </CommandContext.Provider>
    );
}
