import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

export const Route = createFileRoute("/audio-test")({
    component: AudioTest,
});

const TEST_URL = "http://localhost:3060/api/assets/audio/audio/sound_beta_2/voice/char_213_mostma/CN_001.ogg";

interface IEventLog {
    name: string;
    time: number;
    detail?: string;
}

function AudioTest() {
    const audioRef = useRef<HTMLAudioElement | null>(null);
    const [url, setUrl] = useState(TEST_URL);
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [readyState, setReadyState] = useState(0);
    const [networkState, setNetworkState] = useState(0);
    const [events, setEvents] = useState<IEventLog[]>([]);
    const startedAtRef = useRef<number>(0);

    const log = (name: string, detail?: string) => {
        const time = startedAtRef.current ? performance.now() - startedAtRef.current : 0;
        setEvents((prev) => [{ name, time, detail }, ...prev].slice(0, 200));
    };

    useEffect(() => {
        const a = audioRef.current;
        if (!a) return;

        const onPlay = () => {
            setIsPlaying(true);
            log("play");
        };
        const onPause = () => {
            setIsPlaying(false);
            log("pause", `currentTime=${a.currentTime.toFixed(2)} ended=${a.ended}`);
        };
        const onEnded = () => {
            setIsPlaying(false);
            log("ended", `currentTime=${a.currentTime.toFixed(2)}`);
        };
        const onTimeUpdate = () => {
            setCurrentTime(a.currentTime);
            setDuration(a.duration);
        };
        const onError = () => {
            log("error", `code=${a.error?.code} msg=${a.error?.message}`);
        };
        const onLoadStart = () => log("loadstart");
        const onLoadedMetadata = () => log("loadedmetadata", `duration=${a.duration.toFixed(2)}`);
        const onLoadedData = () => log("loadeddata");
        const onCanPlay = () => log("canplay");
        const onCanPlayThrough = () => log("canplaythrough");
        const onWaiting = () => log("waiting", `currentTime=${a.currentTime.toFixed(2)} buffered=${formatBuffered(a)}`);
        const onStalled = () => log("stalled", `currentTime=${a.currentTime.toFixed(2)}`);
        const onSuspend = () => log("suspend");
        const onAbort = () => log("abort");
        const onEmptied = () => log("emptied");
        const onSeeking = () => log("seeking", `currentTime=${a.currentTime.toFixed(2)}`);
        const onSeeked = () => log("seeked", `currentTime=${a.currentTime.toFixed(2)}`);
        const onRateChange = () => log("ratechange");
        const onPlaying = () => log("playing");
        const onProgress = () => {
            setReadyState(a.readyState);
            setNetworkState(a.networkState);
        };
        const onDurationChange = () => log("durationchange", `duration=${a.duration.toFixed(2)}`);

        a.addEventListener("play", onPlay);
        a.addEventListener("pause", onPause);
        a.addEventListener("ended", onEnded);
        a.addEventListener("timeupdate", onTimeUpdate);
        a.addEventListener("error", onError);
        a.addEventListener("loadstart", onLoadStart);
        a.addEventListener("loadedmetadata", onLoadedMetadata);
        a.addEventListener("loadeddata", onLoadedData);
        a.addEventListener("canplay", onCanPlay);
        a.addEventListener("canplaythrough", onCanPlayThrough);
        a.addEventListener("waiting", onWaiting);
        a.addEventListener("stalled", onStalled);
        a.addEventListener("suspend", onSuspend);
        a.addEventListener("abort", onAbort);
        a.addEventListener("emptied", onEmptied);
        a.addEventListener("seeking", onSeeking);
        a.addEventListener("seeked", onSeeked);
        a.addEventListener("ratechange", onRateChange);
        a.addEventListener("playing", onPlaying);
        a.addEventListener("progress", onProgress);
        a.addEventListener("durationchange", onDurationChange);

        return () => {
            a.removeEventListener("play", onPlay);
            a.removeEventListener("pause", onPause);
            a.removeEventListener("ended", onEnded);
            a.removeEventListener("timeupdate", onTimeUpdate);
            a.removeEventListener("error", onError);
            a.removeEventListener("loadstart", onLoadStart);
            a.removeEventListener("loadedmetadata", onLoadedMetadata);
            a.removeEventListener("loadeddata", onLoadedData);
            a.removeEventListener("canplay", onCanPlay);
            a.removeEventListener("canplaythrough", onCanPlayThrough);
            a.removeEventListener("waiting", onWaiting);
            a.removeEventListener("stalled", onStalled);
            a.removeEventListener("suspend", onSuspend);
            a.removeEventListener("abort", onAbort);
            a.removeEventListener("emptied", onEmptied);
            a.removeEventListener("seeking", onSeeking);
            a.removeEventListener("seeked", onSeeked);
            a.removeEventListener("ratechange", onRateChange);
            a.removeEventListener("playing", onPlaying);
            a.removeEventListener("progress", onProgress);
            a.removeEventListener("durationchange", onDurationChange);
        };
    }, []);

    const handlePlay = () => {
        const a = audioRef.current;
        if (!a) return;
        startedAtRef.current = performance.now();
        setEvents([]);
        // Cache-bust to force the browser to re-fetch from backend.
        const sep = url.includes("?") ? "&" : "?";
        a.src = `${url}${sep}_=${Date.now()}`;
        a.play().catch((e) => log("play.catch", String(e)));
    };

    const handlePause = () => {
        audioRef.current?.pause();
    };

    const handleResume = () => {
        audioRef.current?.play().catch((e) => log("resume.catch", String(e)));
    };

    const handleStop = () => {
        const a = audioRef.current;
        if (!a) return;
        a.pause();
        a.removeAttribute("src");
        a.load();
    };

    const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

    return (
        <main style={{ maxWidth: 920, margin: "0 auto", padding: 24, fontFamily: "monospace" }}>
            <h1 style={{ fontFamily: "sans-serif", fontSize: 24, marginBottom: 16 }}>Audio playback diagnostic</h1>

            <div
                style={{
                    padding: 16,
                    borderRadius: 8,
                    background: isPlaying ? "#003" : "#300",
                    color: isPlaying ? "#7df" : "#f77",
                    fontSize: 28,
                    fontWeight: 700,
                    textAlign: "center",
                    border: `2px solid ${isPlaying ? "#7df" : "#f77"}`,
                    marginBottom: 16,
                }}
            >
                {isPlaying ? "▶  PLAYING" : "■  STOPPED"}
            </div>

            <div style={{ marginBottom: 16, display: "flex", gap: 8, flexWrap: "wrap" }}>
                <button type="button" onClick={handlePlay} style={btn}>
                    Load + Play
                </button>
                <button type="button" onClick={handlePause} style={btn}>
                    Pause
                </button>
                <button type="button" onClick={handleResume} style={btn}>
                    Resume
                </button>
                <button type="button" onClick={handleStop} style={btn}>
                    Stop / Clear
                </button>
            </div>

            <input type="text" value={url} onChange={(e) => setUrl(e.target.value)} style={{ width: "100%", padding: 8, marginBottom: 16, fontFamily: "monospace", fontSize: 12 }} />

            <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 8, marginBottom: 16, fontSize: 13 }}>
                <Stat label="currentTime" value={currentTime.toFixed(3)} />
                <Stat label="duration" value={Number.isFinite(duration) ? duration.toFixed(3) : String(duration)} />
                <Stat label="readyState" value={`${readyState} (${READY_STATES[readyState] ?? "?"})`} />
                <Stat label="networkState" value={`${networkState} (${NETWORK_STATES[networkState] ?? "?"})`} />
            </div>

            <div style={{ height: 8, background: "#222", borderRadius: 4, marginBottom: 16, overflow: "hidden" }}>
                <div style={{ height: "100%", width: `${progress}%`, background: isPlaying ? "#7df" : "#666", transition: "width 0.1s" }} />
            </div>

            <h2 style={{ fontFamily: "sans-serif", fontSize: 16, marginBottom: 8 }}>Event log (newest first)</h2>
            <div
                style={{
                    background: "#111",
                    color: "#ddd",
                    padding: 12,
                    borderRadius: 8,
                    height: 400,
                    overflowY: "auto",
                    fontSize: 12,
                    lineHeight: 1.5,
                }}
            >
                {events.length === 0 && <div style={{ opacity: 0.5 }}>No events yet — click "Load + Play"</div>}
                {events.map((evt, i) => (
                    <div key={`${evt.time}-${i}`} style={{ display: "flex", gap: 12 }}>
                        <span style={{ color: "#666", minWidth: 70 }}>{evt.time.toFixed(0).padStart(6)}ms</span>
                        <span style={{ color: COLOR_FOR_EVENT[evt.name] ?? "#ddd", minWidth: 140 }}>{evt.name}</span>
                        {evt.detail && <span style={{ color: "#999" }}>{evt.detail}</span>}
                    </div>
                ))}
            </div>

            <audio ref={audioRef} preload="auto" />
        </main>
    );
}

function Stat({ label, value }: { label: string; value: string }) {
    return (
        <div style={{ background: "#1a1a1a", padding: 8, borderRadius: 4, color: "#ddd" }}>
            <div style={{ fontSize: 10, color: "#888", textTransform: "uppercase" }}>{label}</div>
            <div>{value}</div>
        </div>
    );
}

function formatBuffered(a: HTMLAudioElement): string {
    const ranges: string[] = [];
    for (let i = 0; i < a.buffered.length; i++) {
        ranges.push(`[${a.buffered.start(i).toFixed(2)}-${a.buffered.end(i).toFixed(2)}]`);
    }
    return ranges.join(", ") || "(none)";
}

const btn: React.CSSProperties = {
    padding: "8px 16px",
    background: "#222",
    color: "#fff",
    border: "1px solid #444",
    borderRadius: 4,
    cursor: "pointer",
    fontFamily: "inherit",
};

const READY_STATES: Record<number, string> = {
    0: "HAVE_NOTHING",
    1: "HAVE_METADATA",
    2: "HAVE_CURRENT_DATA",
    3: "HAVE_FUTURE_DATA",
    4: "HAVE_ENOUGH_DATA",
};

const NETWORK_STATES: Record<number, string> = {
    0: "NETWORK_EMPTY",
    1: "NETWORK_IDLE",
    2: "NETWORK_LOADING",
    3: "NETWORK_NO_SOURCE",
};

const COLOR_FOR_EVENT: Record<string, string> = {
    play: "#7df",
    playing: "#7df",
    pause: "#f77",
    ended: "#f77",
    error: "#f33",
    waiting: "#fa3",
    stalled: "#fa3",
    suspend: "#fa3",
    abort: "#f33",
    loadstart: "#7f7",
    loadedmetadata: "#7f7",
    canplay: "#7f7",
    canplaythrough: "#7f7",
};
