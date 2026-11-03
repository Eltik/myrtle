import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Tooltip, TooltipPopup, TooltipTrigger } from "./tooltip";

function mountOpen() {
    return render(
        <Tooltip open>
            <TooltipTrigger render={<button type="button" />}>trigger</TooltipTrigger>
            <TooltipPopup>hint</TooltipPopup>
        </Tooltip>,
    );
}

afterEach(() => {
    cleanup();
    Object.defineProperty(document, "fullscreenElement", { configurable: true, value: null });
    document.body.replaceChildren();
});

describe("TooltipPopup", () => {
    it("portals to the body when nothing is fullscreen", () => {
        mountOpen();
        expect(document.body.contains(screen.getByText("hint"))).toBe(true);
    });

    it("joins the fullscreen element, where a body portal would be hidden", () => {
        const stage = document.createElement("div");
        document.body.append(stage);
        Object.defineProperty(document, "fullscreenElement", { configurable: true, value: stage });
        mountOpen();
        expect(stage.contains(screen.getByText("hint"))).toBe(true);
    });
});

function tap(element: Element, pointerType = "touch") {
    act(() => {
        fireEvent.pointerDown(element, { pointerType });
        fireEvent.pointerUp(element, { pointerType });
        fireEvent.click(element);
    });
}

describe("TooltipTrigger touch path", () => {
    it("a tap on a trigger with no action of its own opens the hint, a second tap closes it", async () => {
        render(
            <Tooltip>
                <TooltipTrigger render={<span />}>label</TooltipTrigger>
                <TooltipPopup>hint</TooltipPopup>
            </Tooltip>,
        );
        expect(screen.queryByText("hint")).toBeNull();
        tap(screen.getByText("label"));
        expect(await screen.findByText("hint")).toBeTruthy();
        expect(screen.getByText("label").hasAttribute("data-popup-open")).toBe(true);
        tap(screen.getByText("label"));
        expect(screen.getByText("label").hasAttribute("data-popup-open")).toBe(false);
    });

    it("a button with its own click keeps it, and the hint stays shut", () => {
        const onClick = vi.fn();
        render(
            <Tooltip>
                <TooltipTrigger render={<button type="button" onClick={onClick} />}>act</TooltipTrigger>
                <TooltipPopup>hint</TooltipPopup>
            </Tooltip>,
        );
        tap(screen.getByText("act"));
        expect(onClick).toHaveBeenCalledTimes(1);
        expect(screen.queryByText("hint")).toBeNull();
    });

    it("a mouse click and a keyboard click never toggle it", () => {
        render(
            <Tooltip>
                <TooltipTrigger render={<span />}>label</TooltipTrigger>
                <TooltipPopup>hint</TooltipPopup>
            </Tooltip>,
        );
        tap(screen.getByText("label"), "mouse");
        act(() => {
            fireEvent.click(screen.getByText("label"));
        });
        expect(screen.queryByText("hint")).toBeNull();
    });
});
