import type { ColumnDef } from "@tanstack/react-table";
import { ArrowUpDown } from "lucide-react";
import Image from "next/image";
import { getAvatar } from "~/helper";
import type { User } from "~/types/impl/api";
import { Button } from "../ui/button";
import { Progress } from "../ui/progress";
import { CharactersDataTable } from "./components/characters/characters-data-table";

function CharactersList({ data }: { data: User }) {
    const columns: ColumnDef<{
        icon: string;
        name: string;
        promotion: number;
        potential: number;
        recruited: number;
        trust: number;
        level: number;
    } | null>[] = [
        {
            accessorKey: "icon",
            header: () => "Icon",
            cell: ({ row }) => {
                return (
                    <Image
                        alt={"Item icon"}
                        height={32}
                        src={row.getValue("icon")}
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
            cell: ({ row }) => {
                return <div className="pl-4">{row.getValue("name")}</div>;
            },
        },
        {
            accessorKey: "promotion",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Promotion
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return (
                    <div className="pl-8">
                        <Image
                            alt="Promotion"
                            height={35}
                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${String(row.getValue("promotion"))}.png`}
                            style={{
                                maxWidth: "100%",
                                height: "auto",
                            }}
                            width={35}
                        />
                    </div>
                );
            },
        },
        {
            accessorKey: "potential",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Potential
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return (
                    <div className="pl-6">
                        <Image
                            alt="Potential"
                            height={40}
                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${Number(row.getValue("potential")) + 1}.png`}
                            style={{
                                maxWidth: "100%",
                                height: "auto",
                            }}
                            width={40}
                        />
                    </div>
                );
            },
        },
        {
            accessorKey: "recruited",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Recruited
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return <p>{new Date(Number(row.getValue("recruited")) * 1000).toLocaleString()}</p>;
            },
        },
        {
            accessorKey: "level",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Level
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return <div className="pl-6">{row.getValue("level")}</div>;
            },
        },
        {
            accessorKey: "trust",
            header: "Trust",
            cell: ({ row }) => {
                return (
                    <div className="flex flex-row items-center gap-3">
                        <Progress className="w-[60%]" value={Number(row.getValue("trust")) / 2} />
                        {row.getValue("trust")}%
                    </div>
                );
            },
        },
    ];

    return (
        <CharactersDataTable
            columns={columns}
            data={Object.values(data.troop.chars).map((char) => {
                return {
                    name: char.static?.name ?? "",
                    promotion: char.evolvePhase,
                    potential: char.potentialRank,
                    recruited: char.gainTime,
                    trust: char.static?.trust ?? 0,
                    icon: getAvatar(char),
                    ...char,
                };
            })}
        />
    );
}

export default CharactersList;
