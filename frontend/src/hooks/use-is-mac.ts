import { useEffect, useState } from "react";

export function useIsMac() {
    const [isMac, setIsMac] = useState(false);
    useEffect(() => {
        setIsMac(/Mac|iPhone|iPad|iPod/i.test(navigator.userAgent));
    }, []);

    return isMac;
}
