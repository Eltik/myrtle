import { useEffect, useRef, useState } from "react";

export const Waveform = ({ audioUrl, isPlaying, onSeek, audioRef }: { audioUrl: string; isPlaying: boolean; onSeek: (time: number) => void; audioRef: React.RefObject<HTMLAudioElement> }) => {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const [audioBuffer, setAudioBuffer] = useState<AudioBuffer | null>(null);
    const [duration, setDuration] = useState<number>(0);
    const [currentTime, setCurrentTime] = useState<number>(0);
    const audioContextRef = useRef<AudioContext | null>(null);
    const animationRef = useRef<number>(0);

    // Initialize audio context and load audio data
    useEffect(() => {
        if (audioUrl === "#") return;

        const initAudio = async () => {
            try {
                // Create audio context if it doesn't exist
                if (!audioContextRef.current) {
                    // Properly type the AudioContext
                    const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext?: AudioContext["constructor"] }).webkitAudioContext;

                    if (AudioContextClass) {
                        audioContextRef.current = new AudioContextClass();
                    } else {
                        console.error("AudioContext not supported in this browser");
                        return;
                    }
                }

                const response = await fetch(audioUrl);
                const arrayBuffer = await response.arrayBuffer();
                if (audioContextRef.current) {
                    const buffer = await audioContextRef.current.decodeAudioData(arrayBuffer);
                    setAudioBuffer(buffer);
                    setDuration(buffer.duration);
                }
            } catch (error) {
                console.error("Error loading audio for waveform:", error);
            }
        };

        void initAudio();

        // Store animation frame ID for cleanup
        const animationFrameId = animationRef.current;

        return () => {
            cancelAnimationFrame(animationFrameId);
        };
    }, [audioUrl]);

    // Draw waveform on canvas
    useEffect(() => {
        if (!canvasRef.current || !audioBuffer) return;

        const canvas = canvasRef.current;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Get the canvas dimensions
        const width = canvas.width;
        const height = canvas.height;

        // Clear the canvas
        ctx.clearRect(0, 0, width, height);

        // Get audio data
        const channelData = audioBuffer.getChannelData(0);
        const step = Math.ceil(channelData.length / width);

        // Draw background
        ctx.fillStyle = "#1a1a1a";
        ctx.fillRect(0, 0, width, height);

        // Draw waveform
        ctx.beginPath();
        ctx.lineWidth = 1;
        ctx.strokeStyle = "#4a4a4a";

        for (let i = 0; i < width; i++) {
            const startIdx = i * step;
            let min = 1.0;
            let max = -1.0;

            for (let j = 0; j < step; j++) {
                if (startIdx + j < channelData.length) {
                    const datum = channelData[startIdx + j];
                    if (typeof datum === "number") {
                        if (datum < min) min = datum;
                        if (datum > max) max = datum;
                    }
                }
            }

            const y = height / 2;
            const lineHeight = Math.max(1, (max - min) * height * 0.8);

            ctx.moveTo(i, y - lineHeight / 2);
            ctx.lineTo(i, y + lineHeight / 2);
        }
        ctx.stroke();

        // Draw playback position
        if (duration > 0) {
            const position = (currentTime / duration) * width;
            ctx.beginPath();
            ctx.lineWidth = 2;
            ctx.strokeStyle = "#a885ff";
            ctx.moveTo(position, 0);
            ctx.lineTo(position, height);
            ctx.stroke();
        }
    }, [audioBuffer, currentTime, duration]);

    // Update playback position
    useEffect(() => {
        let frameId = 0;

        const updatePlaybackPosition = () => {
            if (isPlaying) {
                const audioElement = audioRef.current;
                if (audioElement instanceof HTMLAudioElement) {
                    setCurrentTime(audioElement.currentTime);
                    frameId = requestAnimationFrame(updatePlaybackPosition);
                }
            }
        };

        if (isPlaying) {
            frameId = requestAnimationFrame(updatePlaybackPosition);
        }

        return () => {
            cancelAnimationFrame(frameId);
        };
    }, [isPlaying, audioRef]);

    // Handle click for seeking
    const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
        if (!canvasRef.current || !duration) return;

        const rect = canvasRef.current.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        const canvasWidth = canvasRef.current.width;
        const clickPercent = clickX / canvasWidth;
        const seekTime = duration * clickPercent;

        setCurrentTime(seekTime);
        onSeek(seekTime);
    };

    return (
        <div className="relative my-2 h-20 w-full">
            <canvas className="h-full w-full cursor-pointer rounded-sm" height={80} onClick={handleCanvasClick} ref={canvasRef} width={800} />
            {!audioBuffer && audioUrl !== "#" && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/20">
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
                </div>
            )}
        </div>
    );
};
