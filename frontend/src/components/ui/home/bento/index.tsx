import { Community } from "./impl/community";
import { EventsTimeline } from "./impl/events-timeline";
import { GetStarted } from "./impl/get-started";
import { OperatorDatabase } from "./impl/operator-database";
import { Planner } from "./impl/planner";
import { Statistics } from "./impl/statistics";

export function BentoGrid() {
    return (
        <>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <GetStarted />
                <Statistics />
                <OperatorDatabase />
                <Planner />
                <EventsTimeline />
                <Community />
            </div>
        </>
    );
}
