export function Watermark({ text }: { text: string }) {
    return (
        <div className="absolute bottom-0 left-6 text-[hsla(0,0%,71%,0.8)]">
            <div className="flex">
                {text.split("").map((ch, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: static watermark string, never reorders
                    <div key={`${ch}-${i}`} className={i === 0 ? "overflow-visible" : "overflow-hidden"}>
                        <span className="text-[36px] [text-shadow:0_0_10px_rgba(0,0,0,0.8)]">{ch}</span>
                    </div>
                ))}
            </div>
        </div>
    );
}
