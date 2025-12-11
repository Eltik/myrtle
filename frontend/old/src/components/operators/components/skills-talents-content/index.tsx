import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";
import SkillsContent from "./impl/skills-content";
import TalentsContent from "./impl/talents-content";

function SkillsTalentsContent({ operator }: { operator: Operator }) {
    return (
        <>
            <div>
                <div className="p-2 px-4 backdrop-blur-2xl">
                    <span className="text-xl font-bold md:text-3xl">Skills & Talents</span>
                </div>
                <Separator />
                <div className="grid px-3 py-4 md:p-4">
                    <div className="space-y-2 md:col-span-2 md:space-y-8">
                        <SkillsContent operator={operator} />
                        <TalentsContent operator={operator} />
                    </div>
                </div>
            </div>
        </>
    );
}

export default SkillsTalentsContent;
