import { TextLoop } from "~/components/ui/motion-primitives/text-loop";

export { BentoGrid } from "./impl/bento-grid";

export function MainContent() {
    return (
        <section className="mb-12">
            <h1 className="mb-4 text-balance font-bold text-3xl text-foreground md:text-4xl lg:text-5xl">
                Elevate your{" "}
                <TextLoop
                    className="text-primary"
                    interval={3}
                    style={{
                        textShadow: "0 0 20px oklch(0.75 0.15 25 / 0.5), 0 0 40px oklch(0.75 0.15 25 / 0.3), 0 0 60px oklch(0.75 0.15 25 / 0.15)",
                    }}
                >
                    <span>Arknights</span>
                    <span>Operator</span>
                    <span>Farming</span>
                    <span>Recruitment</span>
                </TextLoop>{" "}
                experience.
            </h1>
            <p className="max-w-2xl text-lg text-muted-foreground">An advanced toolkit for the modern Doctor. Track your operators, plan your strategies, and optimize your gameplay.</p>
        </section>
    );
}
