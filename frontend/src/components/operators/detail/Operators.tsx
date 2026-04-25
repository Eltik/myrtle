import { useSuspenseQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { operatorQueryOptions } from "#/lib/api/operators";

export function OperatorDetail() {
    const { id } = useParams({ from: "/operators_/$id" });
    const { data: operator } = useSuspenseQuery(operatorQueryOptions(id));

    return <p>Hello {operator?.name}</p>;
}
