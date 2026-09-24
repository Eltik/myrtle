import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "reader.toolbar.back": {
        text: "Library",
        description: "Toolbar button that leaves the reader for the story library.",
    },
    "reader.toolbar.auto": {
        text: "Auto",
        description: "Toolbar toggle for auto-play (lines advance on their own). Short; sits in a compact toolbar.",
    },
    "reader.toolbar.hide": {
        text: "Hide",
        description: "Toolbar toggle that takes the text box and every control away and leaves only the scene art. Any tap or key brings them back.",
    },
    "reader.toolbar.hideToolbar": {
        text: "Hide toolbar",
        description: "Toolbar button that collapses the reader controls and the progress bar, leaving the text box in place. Remembered.",
    },
    "reader.toolbar.showToolbar": {
        text: "Show toolbar",
        description: "Accessible name of the small handle at the top edge that brings the collapsed reader controls back.",
    },
    "reader.toolbar.skip": {
        text: "Skip",
        description: "Toolbar button that fast-forwards to the next choice or the end of the story. Short; sits in a compact toolbar.",
    },
    "reader.cutscene.label": {
        text: "Cutscene",
        description: "Name of the video layer, and the row a cutscene leaves in the story log.",
    },
    "reader.cutscene.skip": {
        text: "Skip video",
        description: "Button over a playing cutscene that jumps to the line after it.",
    },
    "reader.cutscene.play": {
        text: "Play",
        description: "Button shown when the browser refused to start a cutscene by itself.",
    },
    "reader.toolbar.on": {
        text: "On",
        description: "The state word printed after Auto in the toolbar when auto-play is running.",
    },
    "reader.toolbar.off": {
        text: "Off",
        description: "The state word printed after Auto in the toolbar when auto-play is not running.",
    },
    "reader.toolbar.sceneLabel": {
        text: "Story controls",
        description: "Accessible name of the left cluster of reader controls (library, settings, log, hide, chapter).",
    },
    "reader.toolbar.playbackLabel": {
        text: "Playback controls",
        description: "Accessible name of the right cluster of reader controls (auto, speed, skip, fullscreen, mute).",
    },
    "reader.skip.confirmTitle": {
        text: "Skip ahead?",
        description: "Title of the one-line confirmation shown the first time Skip is pressed in a session.",
    },
    "reader.skip.confirmBody": {
        text: "Lines play instantly until the next choice or the end of the story. Click anywhere to stop.",
        description: "Body of the skip confirmation.",
    },
    "reader.skip.confirm": {
        text: "Skip",
        description: "Button that starts skipping.",
    },
    "reader.skip.cancel": {
        text: "Cancel",
        description: "Button that dismisses the skip confirmation.",
    },
    "reader.toolbar.log": {
        text: "Log",
        description: "Toolbar button opening the backlog of lines read so far.",
    },
    "reader.toolbar.chapter": {
        text: "Chapter",
        description: "Toolbar button opening the jump-to-story dialog.",
    },
    "reader.toolbar.previousStory": {
        text: "Previous story",
        description: "Toolbar arrow going one story back in the same chapter. Disabled on the first story.",
    },
    "reader.toolbar.nextStory": {
        text: "Next story",
        description: "Toolbar arrow going one story forward in the same chapter. Disabled on the last story.",
    },
    "reader.toolbar.settings": {
        text: "Settings",
        description: "Toolbar button opening the reader settings.",
    },
    "reader.toolbar.fullscreen": {
        text: "Fullscreen",
        description: "Toolbar button entering fullscreen.",
    },
    "reader.toolbar.exitFullscreen": {
        text: "Exit fullscreen",
        description: "Toolbar button leaving fullscreen.",
    },
    "reader.toolbar.mute": {
        text: "Mute music",
        description: "Toolbar button muting music and sound.",
    },
    "reader.toolbar.unmute": {
        text: "Unmute music",
        description: "Toolbar button restoring music and sound.",
    },
    "reader.progress": {
        text: "Reading progress",
        description: "Accessible label of the thin progress bar across the top of the reader.",
    },
    "reader.title.begin": {
        text: "Click or press Space to begin",
        description: "Hint on the title card shown before the first line.",
    },
    "reader.title.resume": {
        text: "You left off at line {line} of {total}.",
        description: "Resume prompt body. `{line}` and `{total}` are 1-based line counts.",
    },
    "reader.title.resumeButton": {
        text: "Resume from line {line}",
        description: "Resume prompt button; `{line}` is 1-based.",
    },
    "reader.title.startOver": {
        text: "Start over",
        description: "Resume prompt button that begins the story from the first line.",
    },
    "reader.end.title": {
        text: "End of story",
        description: "Heading of the end card.",
    },
    "reader.end.body": {
        text: "Use Chapter to pick another, or press the left arrow key to look back.",
        description: "End card body. `Chapter` is the toolbar button of that name; the left arrow key steps back through the lines already read.",
    },
    "reader.end.previous": {
        text: "Previous story",
        description: "End card button to the previous story in the same group.",
    },
    "reader.end.next": {
        text: "Next story",
        description: "End card button to the next story in the same group.",
    },
    "reader.end.library": {
        text: "Back to library",
        description: "End card link to the story library.",
    },
    "reader.end.again": {
        text: "Read again",
        description: "End card button restarting the story.",
    },
    "reader.line.continue": {
        text: "Continue",
        description: "Accessible label of the chevron that marks a finished line.",
    },
    "reader.narrator": {
        text: "Narration",
        description: "Speaker label used in the backlog for lines with no speaker.",
    },
    "reader.stage.label": {
        text: "Story stage. Click to advance.",
        description: "Accessible label of the scene area.",
    },
    "reader.error.noScript": {
        text: "This story has no script file in the served data.",
        description: "Shown when the library lists a story but the backend has no script for it.",
    },
    "reader.error.notFound": {
        text: "Story not found.",
        description: "Shown for an unknown story id.",
    },
    "reader.loading": {
        text: "Loading story…",
        description: "Loading state of the reader.",
    },
    "backlog.title": {
        text: "Backlog",
        description: "Title of the dialog listing every line read so far.",
    },
    "backlog.empty": {
        text: "Nothing read yet.",
        description: "Backlog empty state.",
    },
    "backlog.copy": {
        text: "Copy",
        description: "Button copying the backlog as plain text.",
    },
    "backlog.copied": {
        text: "Copied",
        description: "Button label right after a successful copy.",
    },
    "backlog.choice": {
        text: "Choice: {text}",
        description: "Backlog row for a decision the reader made; `{text}` is the chosen option.",
    },
    "chapter.title": {
        text: "Jump to a story",
        description: "Title of the chapter-jump dialog.",
    },
    "chapter.category": {
        text: "Category",
        description: "Label of the category select in the chapter-jump dialog.",
    },
    "chapter.group": {
        text: "Group",
        description: "Label of the story group select (chapter or event).",
    },
    "chapter.operator": {
        text: "Operator",
        description: "Label of the operator select when the category is operator records.",
    },
    "chapter.story": {
        text: "Story",
        description: "Label of the story select.",
    },
    "chapter.open": {
        text: "Open",
        description: "Button that navigates to the selected story.",
    },
    "chapter.noScript": {
        text: "(no script)",
        description: "Suffix on a story option whose script is missing.",
    },
    "settings.title": {
        text: "Reader settings",
        description: "Title of the settings dialog.",
    },
    "settings.cps": {
        text: "Text speed",
        description: "Setting: typewriter speed in characters per second.",
    },
    "settings.cpsValue": {
        text: "{value} chars/s",
        description: "Displayed value of the text speed slider.",
    },
    "settings.textSize": {
        text: "Text size",
        description: "Setting: text size in percent.",
    },
    "settings.percent": {
        text: "{value}%",
        description: "A percentage value.",
    },
    "settings.lineWidth": {
        text: "Line width",
        description: "Setting: the text box's maximum line width.",
    },
    "settings.lineHeight": {
        text: "Line height",
        description: "Setting: the spacing between lines of dialogue, as a multiple of the text size.",
    },
    "settings.lineWidthValue": {
        text: "{value} ch",
        description: "Displayed value of the line width slider; `ch` is the CSS unit (width of a digit).",
    },
    "settings.sideMargin": {
        text: "Side margin",
        description: "Setting: how far the text box is inset from the left and right edges of the stage.",
    },
    "settings.bottomMargin": {
        text: "Bottom margin",
        description: "Setting: how far the text box is lifted off the bottom edge of the stage.",
    },
    "settings.lightBox": {
        text: "Light text box",
        description: "Setting: the text box switches to a light paper surface. Only the reading surface changes, not the site theme.",
    },
    "settings.lightBoxHint": {
        text: "Only the reading surface changes; the site theme stays as you set it.",
        description: "Hint under the light text box switch.",
    },
    "settings.letterbox": {
        text: "Letterbox like the game",
        description: "Setting: the area outside the 16:9 scene is painted black, exactly as the game draws it, instead of showing a blurred continuation of the background.",
    },
    "settings.letterboxHint": {
        text: "The game draws every scene into a 16:9 box and fills the rest of the screen with black. Off, the reader continues the background across it, blurred and dimmed.",
        description: "Hint under the letterbox switch.",
    },
    "settings.animateRatio": {
        text: "Playback speed",
        description: "Setting: the multiplier the game applies to every scene animation. 1 is the scripted speed, 0 skips animations.",
    },
    "settings.animateRatioHint": {
        text: "Multiplies every fade, tween and hold the script asks for. 0 plays each step instantly, which is what the game's skip does.",
        description: "Hint under the playback speed slider.",
    },
    "settings.autoPace": {
        text: "Auto-play pace",
        description: "Setting: multiplier on the reading time auto-play waits per line.",
    },
    "settings.multiplier": {
        text: "{value}x",
        description: "A multiplier value.",
    },
    "settings.minLineSec": {
        text: "Minimum line time",
        description: "Setting: the least time auto-play holds a line.",
    },
    "settings.seconds": {
        text: "{value} s",
        description: "A duration in seconds.",
    },
    "settings.musicVolume": {
        text: "Music volume",
        description: "Setting: background music volume.",
    },
    "settings.sfxVolume": {
        text: "Sound volume",
        description: "Setting: sound effect volume.",
    },
    "settings.progressBar": {
        text: "Progress bar",
        description: "Setting: show the thin progress bar at the top of the reader.",
    },
    "settings.playVideos": {
        text: "Play cutscene videos",
        description: "Setting: play the cutscene clip a story's [Video] command names. Off skips every cutscene.",
    },
    "settings.playVideosHint": {
        text: "The URL can turn them off too, with ?video=0.",
        description: "Hint under the cutscene setting, naming the search parameter that does the same thing.",
    },
    "settings.cutscenePlayer": {
        text: "Cutscene player",
        description: "Setting: which controls a playing cutscene carries. Sits under the toggle that turns cutscenes on.",
    },
    "settings.cutscenePlayerHint": {
        text: "Full player loads on the first cutscene; its volume follows the Music slider both ways.",
        description: "Hint under the cutscene player setting: the full player is fetched only when a clip plays, and its volume control writes back to the reader's music volume.",
    },
    "settings.cutscenePlayer.simple": {
        text: "Simple (skip only)",
        description: "Cutscene player option: the clip plays with no controls at all, just the Skip button.",
    },
    "settings.cutscenePlayer.native": {
        text: "Browser controls",
        description: "Cutscene player option: the browser's own video control bar (play, scrub, volume, fullscreen).",
    },
    "settings.cutscenePlayer.vidstack": {
        text: "Full player",
        description: "Cutscene player option: a full video player with its own control bar and settings menu.",
    },
    "settings.nickname": {
        text: "Doctor's name",
        description: "Setting: the name substituted for the player in story text.",
    },
    "settings.nicknameHint": {
        text: "Used wherever a story addresses you by name. Up to 24 characters; an empty field reads as Doctor.",
        description: "Caption under the Doctor's name field.",
    },
    "settings.style": {
        text: "Reading style",
        description: "Setting: a preset of font, size and line height.",
    },
    "settings.styleHint": {
        text: "Sets the font, size, width and spacing below in one go. Changing any of them reads as Custom.",
        description: "Caption under the reading style select. 'Custom' matches the option name in the same select.",
    },
    "settings.font": {
        text: "Font",
        description: "Settings row label for the dialogue font family.",
    },
    "settings.fontHint": {
        text: "Replaces the reading style's typeface only; its size and spacing stay.",
        description: "Hint under the font row.",
    },
    "settings.font.terraCredit": {
        text: "Fan reconstruction of the in-game script; credits in the footer.",
        description: "Note under the font dropdown when one of the Terra script fonts is chosen.",
    },
    "settings.font.previewText": {
        text: "Doctor. The convoy reaches the checkpoint at dawn.",
        description: "Sample sentence under the font dropdown, drawn in the chosen font at the reader's own text size. Any natural sentence of similar length works.",
    },
    "settings.font.upload": {
        text: "Choose font file",
        description: "Button that opens the file picker for a custom font (.ttf, .otf, .woff, .woff2).",
    },
    "settings.font.remove": {
        text: "Remove",
        description: "Button that deletes the uploaded custom font.",
    },
    "settings.font.none": {
        text: "No font file yet",
        description: "Shown beside the upload button when no custom font is stored.",
    },
    "settings.font.rejected": {
        text: "That file is not a font the browser can load.",
        description: "Error shown when an uploaded file is not a usable .ttf, .otf, .woff or .woff2.",
    },
    "settings.font.tooBig": {
        text: "That font file is larger than {mb} MB.",
        description: "Error shown when an uploaded font exceeds the size limit. `{mb}` is a whole number of megabytes.",
    },
    "settings.textColor": {
        text: "Text colour",
        description: "Settings row label for the dialogue and narration colour.",
    },
    "settings.textColorHint": {
        text: "Every swatch clears 4.5:1 on both the dark and the light box. The speaker's name keeps its own colour.",
        description: "Hint under the text colour row.",
    },
    "settings.textColor.default": {
        text: "Default",
        description: "Swatch that restores the box's own text colour.",
    },
    "settings.textColor.swatch": {
        text: "Colour {number}",
        description: "Accessible label of a curated colour swatch. `{number}` is 1-based.",
    },
    "settings.textColor.custom": {
        text: "Custom colour",
        description: "Accessible label of the native colour input beside the swatches.",
    },
    "settings.speakerTint": {
        text: "Speaker colour",
        description: "Settings row label for where the speaker's own colour is applied.",
    },
    "settings.speakerTintHint": {
        text: "The colour is sampled from the speaker's sprite on stage, and a speaker with nobody on stage keeps a hue hashed from their name. Either way it is kept readable at 4.5:1 on the box, and narration keeps the text colour.",
        description: "Hint under the speaker colour row: where the colour comes from and that contrast is preserved.",
    },
    "settings.speakerTint.off": {
        text: "Off (default)",
        description: "Speaker colour option: the curated hue hashed from the speaker's name, with the line in the text colour.",
    },
    "settings.speakerTint.text": {
        text: "Text and name",
        description: "Speaker colour option: the sprite's colour on the dialogue line and on the speaker's name.",
    },
    "settings.speakerTint.name": {
        text: "Name only",
        description: "Speaker colour option: the sprite's colour on the speaker's name, the line keeping the text colour.",
    },
    "settings.boxPosition": {
        text: "Box position",
        description: "Settings row label for where the text box sits on the stage, on both axes.",
    },
    "settings.boxPositionHint": {
        text: "Drag the box on the stage, or drag the thumbnail here. Arrow keys nudge it.",
        description: "Hint under the box position row.",
    },
    "settings.boxPosition.pad": {
        text: "Text box position on the stage",
        description: "Accessible label of the miniature stage in settings that the text box thumbnail is dragged around.",
    },
    "settings.boxPosition.value": {
        text: "{x}, {y}",
        description: "The box position read out beside its row. `{x}` and `{y}` are percentages across and up the stage.",
    },
    "settings.boxPreset.bottomCentre": {
        text: "Bottom centre",
        description: "Text box position preset: the default, centred against the bottom margin.",
    },
    "settings.boxPreset.bottomLeft": {
        text: "Bottom left",
        description: "Text box position preset.",
    },
    "settings.boxPreset.bottomRight": {
        text: "Bottom right",
        description: "Text box position preset.",
    },
    "settings.boxPreset.centre": {
        text: "Centre",
        description: "Text box position preset: halfway up the stage, centred.",
    },
    "settings.boxPreset.topCentre": {
        text: "Top centre",
        description: "Text box position preset: as high as the box goes, centred.",
    },
    "settings.boxPreset.reset": {
        text: "Reset",
        description: "Button restoring the text box to its default bottom-centre position.",
    },
    "settings.autoHideToolbar": {
        text: "Auto-hide toolbar after 2.5 s",
        description: "Settings toggle that fades the reader controls and the progress bar out while you read.",
    },
    "settings.autoHideToolbarHint": {
        text: "Off by default: the controls stay on screen, the way the game keeps its own. With it on, moving the pointer brings them back; reading does not, so Space keeps turning the page.",
        description: "Hint under the auto-hide toolbar row.",
    },
    "settings.toolbarIdle": {
        text: "Hide toolbar after",
        description: "Settings row label for how long the auto-hide waits before the controls fade.",
    },
    "settings.toolbarIdleHint": {
        text: "How long the controls wait with no pointer movement.",
        description: "Hint under the auto-hide delay slider.",
    },
    "settings.toolbarHidden": {
        text: "Hide toolbar",
        description: "Settings toggle that collapses the reader controls and the progress bar while keeping the text box.",
    },
    "settings.toolbarHiddenHint": {
        text: "The text box stays; only the controls and the progress bar go. A handle at the top edge brings them back, T does too, and the choice is remembered.",
        description: "Hint under the hide toolbar row. `T` is a keyboard key and stays as-is.",
    },
    "reader.box.drag": {
        text: "Move the text box",
        description: "Accessible label of the text box's drag handle, which moves the box around the stage.",
    },
    "reader.toolbar.speed": {
        text: "Playback speed",
        description: "Label of the 1x/2x playback speed toggle in the reader's right control cluster.",
    },
    "reader.scrub.line": {
        text: "Line {line} of {total}",
        description: "First line of the progress bar tooltip. `{line}` is 1-based, `{total}` the story's line count.",
    },
    "backlog.jump": {
        text: "Jump to line {line}",
        description: "Title of a backlog row, which replays the story to that line. `{line}` is 1-based.",
    },
    "settings.progress.heading": {
        text: "Reading progress",
        description: "Heading of the backup, restore and reset section.",
    },
    "settings.progress.backup": {
        text: "Backup",
        description: "Button downloading the reading progress as a JSON file.",
    },
    "settings.progress.restore": {
        text: "Restore",
        description: "Button choosing a JSON backup file to restore progress from.",
    },
    "settings.progress.reset": {
        text: "Reset",
        description: "Button clearing all reading progress (asks for confirmation).",
    },
    "settings.progress.restored": {
        text: "Progress restored: {stories} stories read, {positions} positions.",
        description: "Confirmation after a backup file was loaded.",
    },
    "settings.progress.restoreFailed": {
        text: "That file is not a progress backup.",
        description: "Error after a backup file could not be read.",
    },
    "settings.reset.title": {
        text: "Reset reading progress?",
        description: "Confirmation dialog title.",
    },
    "settings.reset.body": {
        text: "Every read mark and saved position is removed. This cannot be undone.",
        description: "Confirmation dialog body.",
    },
    "settings.reset.confirm": {
        text: "Reset",
        description: "Confirmation dialog destructive button.",
    },
    "settings.reset.cancel": {
        text: "Cancel",
        description: "Confirmation dialog cancel button.",
    },
    "settings.hotkeys": {
        text: "Space, Enter or → advance · ← back · [ and ] previous and next story · A auto · S skip · H or L log · Esc hide everything · T hide the toolbar only (remembered) · F fullscreen · M mute",
        description: "One-line hotkey summary at the bottom of the settings dialog. The letters are keyboard keys and stay as-is.",
    },
    "settings.hotkeys.heading": {
        text: "Keyboard",
        description: "Heading of the hotkey legend in the settings dialog.",
    },
    "settings.progress.wasReset": {
        text: "Reading progress cleared.",
        description: "Confirmation shown in the settings dialog after the reset was confirmed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
