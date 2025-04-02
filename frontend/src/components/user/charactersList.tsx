import type { ColumnDef } from "@tanstack/react-table";
import Image from "next/image";
import type { User } from "~/types/impl/api";
import { Button } from "../ui/button";
import { ArrowUpDown } from "lucide-react";
import { CharactersDataTable } from "./components/characters/characters-data-table";
import { Progress } from "../ui/progress";
import { getAvatar } from "~/helper";

function CharactersList({ data }: { data: User }) {
    const columns: ColumnDef<{
        icon: string;
        name: string;
        promotion: number;
        potential: number;
        recruited: number;
        trust: number;
    } | null>[] = [
        {
            accessorKey: "icon",
            header: () => "Icon",
            cell: ({ row }) => {
                return (
                    <Image
                        src={row.getValue("icon")}
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
            cell: ({ row }) => {
                return <div className="pl-4">{row.getValue("name")}</div>;
            },
        },
        {
            accessorKey: "promotion",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Promotion
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return (
                    <div className="pl-8">
                        <Image
                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${String(row.getValue("promotion"))}.png`}
                            width={35}
                            height={35}
                            alt="Promotion"
                            style={{
                                maxWidth: "100%",
                                height: "auto",
                            }}
                        />
                    </div>
                );
            },
        },
        {
            accessorKey: "potential",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
                        Potential
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                return (
                    <div className="pl-6">
                        <Image
                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${Number(row.getValue("potential")) + 1}.png`}
                            width={40}
                            height={40}
                            alt="Potential"
                            style={{
                                maxWidth: "100%",
                                height: "auto",
                            }}
                        />
                    </div>
                );
            },
        },
        {
            accessorKey: "recruited",
            header: ({ column }) => {
                return (
                    <Button variant="ghost" onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
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
            accessorKey: "trust",
            header: "Trust",
            cell: ({ row }) => {
                return (
                    <div className="flex flex-row items-center gap-3">
                        <Progress value={Number(row.getValue("trust")) / 2} className="w-[60%]" />
                        {row.getValue("trust")}%
                    </div>
                );
            },
        },
    ];

    return (
        <>
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
        </>
    );
}

export default CharactersList;
