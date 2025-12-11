import type { ColumnDef } from "@tanstack/react-table";
import { ArrowUpDown } from "lucide-react";
import Image from "next/image";
import type { User } from "~/types/impl/api";
import type { Item } from "~/types/impl/api/static/material";
import { Button } from "../ui/button";
import { ItemsDataTable } from "./components/items/items-data-table";

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
                        alt={"Item icon"}
                        height={32}
                        src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${String(row.getValue("icon"))}.png`}
                        style={{
                            maxWidth: "100%",
                            height: "auto",
                        }}
                        width={32}
                    />
                );
            },
        },
        {
            accessorKey: "name",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
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
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
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
    );
}

export default Items;
