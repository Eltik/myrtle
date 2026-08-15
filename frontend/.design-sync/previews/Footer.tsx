import { Footer } from "frontend";

export const Default = () => <Footer />;

export const BelowPageContent = () => (
    <div className="flex flex-col">
        <section className="mx-auto w-[min(1080px,calc(100%-2rem))] pb-10">
            <span className="mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]">Operator index</span>
            <h2 className="m-0 mb-3 font-bold font-sans text-[28px] text-foreground leading-[1.05] tracking-[-0.03em]">438 operators, every skill and module.</h2>
            <p className="m-0 max-w-[60ch] font-sans text-muted-foreground">Recomputed from Hypergryph&rsquo;s gamedata on every build. Skills, modules, skins, stages and enemies are mirrored for both the Global and CN releases.</p>
        </section>
        <Footer />
    </div>
);
