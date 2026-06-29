export function Watermark({ text }: { text: string }) {
    return (
        <div className="pointer-events-none absolute bottom-0 left-3 select-none text-[hsla(0,0%,71%,0.8)] sm:left-6" aria-hidden="true">
            <div className="flex">
                {text.split("").map((ch, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: static watermark string, never reorders
                    <div key={`${ch}-${i}`} className={i === 0 ? "overflow-visible" : "overflow-hidden"}>
                        <span className="text-[26px] [text-shadow:0_0_10px_rgba(0,0,0,0.8)] sm:text-[36px]">{ch}</span>
                    </div>
                ))}
            </div>
        </div>
    );
}
