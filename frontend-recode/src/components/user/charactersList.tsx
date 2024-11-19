import type { ColumnDef } from "@tanstack/react-table";
import Image from "next/image";
import type { CharacterData, User } from "~/types/impl/api";
import { Button } from "../ui/button";
import { ArrowUpDown } from "lucide-react";
import { CharactersDataTable } from "./components/characters-data-table";
import { Progress } from "../ui/progress";

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
                return <Image src={row.getValue("icon")} width={32} height={32} alt={"Item icon"} />;
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
                return <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${row.getValue("promotion")}.png`} width={35} height={35} alt="Promotion" />;
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
                return <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${(row.getValue("potential") as number) + 1}.png`} width={40} height={40} alt="Potential" />;
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
                return <p>{new Date((row.getValue("recruited") as number) * 1000).toLocaleString()}</p>;
            },
        },
        {
            accessorKey: "trust",
            header: "Trust",
            cell: ({ row }) => {
                return (
                    <div className="flex flex-row items-center gap-3">
                        <Progress value={(row.getValue("trust") as number) / 2} className="w-[60%]" />
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
                    let skinId = "";

                    const normalizeSkinId = (skinId: string) => {
                        if (skinId.includes("@")) {
                            return encodeURIComponent(skinId.replaceAll("@", "_"));
                        } else {
                            return encodeURIComponent(skinId.replaceAll("#", "_"));
                        }
                    };

                    if (!char.skin || char.skin.endsWith("#1")) {
                        skinId = normalizeSkinId(char.charId);
                    } else if (char.skin.endsWith("#2")) {
                        skinId = normalizeSkinId(char.charId) + "_2";
                    } else if (char.skin.includes("@")) {
                        skinId = normalizeSkinId(char.skin);
                    } else {
                        skinId = normalizeSkinId(char.charId);
                    }

                    const icon = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${skinId}.png`;
                    return {
                        name: char.static?.name ?? "",
                        promotion: char.evolvePhase,
                        potential: char.potentialRank,
                        recruited: char.gainTime,
                        trust: char.static?.trust ?? 0,
                        icon: skinId.length === 0 ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${normalizeSkinId(char.charId)}.png` : icon,
                        ...char,
                    };
                })}
            />
        </>
    );
}

export default CharactersList;
