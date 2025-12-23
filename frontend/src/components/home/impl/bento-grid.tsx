import { Community } from "./community";
import { EventsTimeline } from "./events-timeline";
import { GetStarted } from "./get-started";
import { OperatorDatabase } from "./operator-database";
import { Planner } from "./planner";
import { Statistics } from "./statistics";

export function BentoGrid() {
    return (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            <GetStarted />
            <Statistics />
            <OperatorDatabase />
            <Planner />
            <EventsTimeline />
            <Community />
        </div>
    );
}
