import type { User } from "~/types/impl/api";
import { ItemsDataTable } from "./components/items/items-data-table";
import type { ColumnDef } from "@tanstack/react-table";
import type { Item } from "~/types/impl/api/static/material";
import Image from "next/image";
import { Button } from "../ui/button";
import { ArrowUpDown } from "lucide-react";

function Items({ data }: { data: User }) {
    const columns: ColumnDef<
        | (Item & {
              amount: number;
          })
        | null
    >[] = [
        {
            accessorKey: "icon",
            header: () => "Icon",
            cell: ({ row }) => {
                return (
                    <Image
                        src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${String(row.getValue("icon"))}.png`}
                        width={32}
                        height={32}
                        alt={"Item icon"}
                        style={{
                            maxWidth: "100%",
                            height: "auto",
                        }}
                    />
                );
            },
        },
        {
            accessorKey: "name",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Name
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
        {
            accessorKey: "amount",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Amount
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return <div className="pl-6">{row.getValue("amount")}</div>;
            },
        },
    ];

    return (
        <>
            <ItemsDataTable
                columns={columns}
                data={Object.values(data.inventory)
                    .map((item) => {
                        if (!item.iconId) return null;
                        return {
                            ...item,
                            icon: item.iconId,
                        };
                    })
                    .filter(Boolean)}
            />
        </>
    );
}

export default Items;
