import type { Operator } from "~/types/impl/api/static/operator";

function LevelUpContent({ operator }: { operator: Operator }) {
    return <div>Level-Up Cost content for {operator.name}</div>;
}

export default LevelUpContent;
