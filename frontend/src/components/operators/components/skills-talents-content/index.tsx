import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";

function SkillsContent({ operator }: { operator: Operator }) {
    return (
        <>
            <div>
                <div className="p-2 px-4 backdrop-blur-2xl">
                    <span className="text-3xl font-bold">Skills & Talents</span>
                </div>
                <Separator />
                <div className="px-3 md:p-4"></div>
            </div>
        </>
    );
}

export default SkillsContent;
