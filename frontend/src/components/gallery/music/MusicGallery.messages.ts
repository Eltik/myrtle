import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gallery";

export const messages = {
    "music.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb trail at the top of the music page.",
    },
    "music.breadcrumb.gallery": {
        text: "Gallery",
        description: "First crumb of the music page: the site's gallery section.",
    },
    "music.title": {
        text: "Music",
        description: "Heading of the music page: Monster Siren Records releases and the game's own soundtrack.",
    },
    "music.search.placeholder": {
        text: "Search by title or event",
        description: "Placeholder in the search box of the From the game view on the music page.",
    },
    "music.search.aria": {
        text: "Search tracks",
        description: "Accessible name of the search box in the music library.",
    },
    "music.kind.aria": {
        text: "What the track is for",
        description: "Accessible name of the strip of buttons that filters tracks by what they play over.",
    },
    "music.kind.all": {
        text: "All",
        description: "Filter button that shows every track.",
    },
    "music.kind.battle": {
        text: "Battle",
        description: "Filter button for music that plays during a fight.",
    },
    "music.kind.lobby": {
        text: "Lobby",
        description: "Filter button for music that plays on the home screen, in menus and on event pages.",
    },
    "music.kind.story": {
        text: "Story",
        description: "Filter button for music that plays in story scenes.",
    },
    "music.kind.map": {
        text: "Map",
        description: "Filter button for music that plays on Integrated Strategies and Reclamation Algorithm maps.",
    },
    "music.results": {
        text: "{count, plural, one {# track} other {# tracks}}",
        description: "How many tracks match the search and filter, shown beside the filters.",
    },
    "music.empty": {
        text: "No tracks match that.",
        description: "Shown in place of the list when the search and filter leave nothing.",
    },
    "music.failed": {
        text: "Couldn't load the music list. Try again in a bit.",
        description: "Shown in place of the list when the request for it fails.",
    },
    "music.unavailableRoute": {
        text: "The music list isn't available yet.",
        description: "Shown when the backend does not serve the music list at all (an older backend).",
    },
    "music.loading": {
        text: "Loading tracks",
        description: "Accessible name of the placeholder rows shown while the list loads.",
    },
    "music.showMore": {
        text: "Show {count} more ({remaining} left)",
        description: "Button under the track list that reveals the next page. {count} is how many it adds, {remaining} how many are still hidden.",
    },
    "music.row.play": {
        text: "Play {name}",
        description: "Accessible name of the play button on a track row.",
    },
    "music.row.pause": {
        text: "Pause {name}",
        description: "Accessible name of the play button on a track row while that track is playing.",
    },
    "music.row.unavailable": {
        text: "Unavailable",
        description: "Badge on a track the game names but whose audio is not published anymore.",
    },
    "music.row.unavailableNote": {
        text: "The game still names this track, but the audio isn't published anymore.",
        description: "Line in an unavailable track's details explaining why it cannot be played.",
    },
    "music.row.shared": {
        text: "Shared",
        description: "Shown where a track's origin would be when it was not made for any one event or mode.",
    },
    "music.row.otherServer": {
        text: "Audio from the {server} client",
        description: "Note on a track whose audio is served from another server's game files. {server} is a server code such as CN.",
    },
    "music.row.details": {
        text: "Where {name} plays",
        description: "Accessible name of the button that expands a track row to show where it plays.",
    },
    "music.row.places": {
        text: "Where it plays",
        description: "Heading of the list of places inside an expanded track row.",
    },
    "music.row.noPlace": {
        text: "Nothing we can trace plays this one. It's still named in these audio banks:",
        description: "Shown in an expanded track row when no stage, story, event or screen is found for it.",
    },
    "music.row.banks": {
        text: "Audio banks",
        description: "Label over the raw audio bank names an expanded track row lists.",
    },
    "music.row.more": {
        text: "+{count} more",
        description: "Button after a shortened list of stages or stories in a track row; it reveals the rest. {count} is how many are hidden.",
    },
    "music.download.label": {
        text: "Download",
        description: "Label before the intro and loop download buttons in an expanded track row.",
    },
    "music.download.intro": {
        text: "Intro",
        description: "Button that downloads the one-shot opening part of a track.",
    },
    "music.download.loop": {
        text: "Loop",
        description: "Button that downloads the looping part of a track.",
    },
    "music.download.introAria": {
        text: "Download the intro of {name}",
        description: "Accessible name of the intro download button.",
    },
    "music.download.loopAria": {
        text: "Download the loop of {name}",
        description: "Accessible name of the loop download button.",
    },
    "music.download.failed": {
        text: "Download failed",
        description: "Shown on a download button when fetching the clip failed.",
    },
    "music.loopNote": {
        text: "Plays the intro once, then loops, like the game does.",
        description: "Short line under the filters explaining how the preview player plays a track.",
    },
    "music.place.stage": {
        text: "Stages",
        description: "Heading of the stages a track plays in, inside an expanded track row.",
    },
    "music.place.story": {
        text: "Story",
        description: "Heading of the story scenes a track plays in.",
    },
    "music.place.season": {
        text: "Contingency Contract",
        description: "Heading of a Contingency Contract season a track plays in.",
    },
    "music.place.event": {
        text: "Event",
        description: "Heading of an event whose menu or minigame plays a track.",
    },
    "music.place.chapter": {
        text: "Main story",
        description: "Heading of a main story chapter whose menu plays a track.",
    },
    "music.place.lobby": {
        text: "Lobby",
        description: "Heading of a home screen or base spot that plays a track.",
    },
    "music.place.map": {
        text: "Map",
        description: "Heading of an Integrated Strategies or Reclamation Algorithm screen that plays a track.",
    },
    "music.role.menu": {
        text: "menu",
        description: "What a track does at a place: it plays on that place's menu or page.",
    },
    "music.role.battle": {
        text: "permanent map",
        description: "What a track does in a Contingency Contract season or Reclamation Algorithm run: it plays in the fights.",
    },
    "music.role.challenge": {
        text: "high risk",
        description: "What a track does in a Contingency Contract season: it plays once the risk passes the season's challenge point.",
    },
    "music.role.minigame": {
        text: "minigame",
        description: "What a track does in an event: it plays in one of the event's minigames or modes.",
    },
    "music.role.rerun": {
        text: "rerun menu",
        description: "What a track does in an event: it plays on the menu of the event's permanent rerun.",
    },
    "music.role.background": {
        text: "home background",
        description: "What a track does in the lobby: a home background plays it.",
    },
    "music.role.jukebox": {
        text: "base jukebox",
        description: "What a track does in the lobby: the base jukebox can play it.",
    },
    "music.role.furniture": {
        text: "base furniture",
        description: "What a track does in the lobby: a piece of base furniture plays it.",
    },
    "music.role.map": {
        text: "node map",
        description: "What a track does on an Integrated Strategies or Reclamation Algorithm screen: it plays on the node map.",
    },
    "music.role.boss": {
        text: "boss floor",
        description: "What a track does in Integrated Strategies: it plays on the map before a boss.",
    },
    "music.role.shop": {
        text: "shop",
        description: "What a track does in Integrated Strategies: it plays in the shop.",
    },
    "music.role.encounter": {
        text: "encounter",
        description: "What a track does in Integrated Strategies: it plays on an encounter node.",
    },
    "music.counts": {
        text: "{albums, plural, one {# album or EP} other {# albums and EPs}} from Monster Siren Records, plus {tracks, plural, one {# track} other {# tracks}} from the game",
        description: "Line under the music page heading once both catalogues have loaded.",
    },
    "music.counts.sirenOnly": {
        text: "{albums, plural, one {# album or EP} other {# albums and EPs}} from Monster Siren Records",
        description: "Line under the music page heading when the label catalogue loaded but the game's music list did not.",
    },
    "music.counts.loading": {
        text: "Loading…",
        description: "Line under the music page heading while the label catalogue loads.",
    },
    "music.counts.syncing": {
        text: "Syncing the label catalogue… This takes a minute the first time the server starts.",
        description: "Shown while the backend is still copying the Monster Siren Records catalogue for the first time; the page retries on its own.",
    },
    "music.counts.noSiren": {
        text: "The server doesn't have the Monster Siren Records catalogue yet, so only the game's {tracks, plural, one {# track} other {# tracks}} are here.",
        description: "Line under the music page heading when the running backend has no label catalogue route.",
    },
    "music.siren.missing": {
        text: "The server doesn't have the Monster Siren Records catalogue yet.",
        description: "Shown in place of label albums and songs when the running backend has no label catalogue route.",
    },
    "music.siren.failed": {
        text: "Couldn't load the Monster Siren Records catalogue. Try again in a bit.",
        description: "Shown when the request for the label catalogue fails.",
    },
    "music.tab.aria": {
        text: "Music sections",
        description: "Accessible name of the Discovery / Library switch.",
    },
    "music.tab.discovery": {
        text: "Discovery",
        description: "Tab that shows the album coverflow.",
    },
    "music.tab.library": {
        text: "Library",
        description: "Tab that shows the browsable library: albums, songs, artists, playlists.",
    },
    "music.tab.lyrics": {
        text: "Lyrics",
        description: "Button that opens or closes the lyrics panel for the current track.",
    },
    "music.tab.lyricsTitle": {
        text: "Show lyrics for the current track",
        description: "Tooltip of the Lyrics button.",
    },
    "music.tab.queue": {
        text: "Queue",
        description: "Button that opens or closes the play queue panel.",
    },
    "music.tab.queueTitle": {
        text: "Show the queue",
        description: "Tooltip of the Queue button.",
    },
    "music.discovery.aria": {
        text: "Albums, newest first",
        description: "Accessible name of the album coverflow.",
    },
    "music.discovery.roledescription": {
        text: "coverflow",
        description: "Screen reader role description of the album carousel. Lowercase, a noun.",
    },
    "music.discovery.current": {
        text: "{name}, album {position} of {total}",
        description: "Accessible name of the coverflow naming the album in the centre.",
    },
    "music.discovery.hint.browse": {
        text: "Drag or scroll to browse",
        description: "Hint under the coverflow.",
    },
    "music.discovery.hint.step": {
        text: "← → to step",
        description: "Hint under the coverflow: the arrow keys move one album.",
    },
    "music.discovery.hint.open": {
        text: "Enter or click to open",
        description: "Hint under the coverflow: Enter or a click on the centre cover opens the album.",
    },
    "music.discovery.jump": {
        text: "Jump to album",
        description: "Link under the coverflow that opens a search palette over album names.",
    },
    "music.discovery.jump.placeholder": {
        text: "Jump to album…",
        description: "Placeholder of the album search palette.",
    },
    "music.discovery.jump.empty": {
        text: "No matches.",
        description: "Shown in the album search palette when nothing matches.",
    },
    "music.discovery.toGame": {
        text: "Browse the game's soundtrack",
        description: "Button under the coverflow's empty state that opens the From the game view.",
    },
    "music.rail.aria": {
        text: "Library sections",
        description: "Accessible name of the library's side menu.",
    },
    "music.rail.albums": {
        text: "Albums",
        description: "Library section: every album and EP.",
    },
    "music.rail.songs": {
        text: "All Songs",
        description: "Library section: every label track.",
    },
    "music.rail.artists": {
        text: "Artists",
        description: "Library section: guest artists.",
    },
    "music.rail.liked": {
        text: "Liked Songs",
        description: "Library section: tracks the listener liked.",
    },
    "music.rail.history": {
        text: "Recently Played",
        description: "Library section: tracks played lately.",
    },
    "music.rail.stats": {
        text: "Your Stats",
        description: "Library section: listening statistics.",
    },
    "music.rail.game": {
        text: "From the game",
        description: "Library section: the game's own soundtrack. Also the queue source name when playing from it.",
    },
    "music.rail.artistView": {
        text: "Artist: {name}",
        description: "Heading of the song list of one artist.",
    },
    "music.rail.playlists": {
        text: "Playlists",
        description: "Heading of the playlist list in the library menu.",
    },
    "music.rail.noPlaylists": {
        text: "No playlists yet",
        description: "Shown under Playlists when there are none.",
    },
    "music.filter.reverse": {
        text: "Reverse sort direction",
        description: "Button that flips the sort order.",
    },
    "music.filter.sortBy": {
        text: "Sort by",
        description: "Accessible name of the sort menu.",
    },
    "music.filter.sort.album.newest": {
        text: "Newest first",
        description: "Album sort: the label's release order.",
    },
    "music.filter.sort.album.title": {
        text: "Title",
        description: "Album sort: alphabetical.",
    },
    "music.filter.sort.album.tracks": {
        text: "Track count",
        description: "Album sort: most tracks first.",
    },
    "music.filter.sort.song.catalog": {
        text: "Catalog order",
        description: "Song sort: the list's own order.",
    },
    "music.filter.sort.song.title": {
        text: "Title",
        description: "Song sort: alphabetical by title.",
    },
    "music.filter.sort.song.album": {
        text: "Album",
        description: "Song sort: by album name.",
    },
    "music.filter.sort.song.artist": {
        text: "Artist",
        description: "Song sort: by artist name.",
    },
    "music.filter.searchPlaceholder": {
        text: "Search tracks, albums, artists…",
        description: "Placeholder of the library search box.",
    },
    "music.filter.filters": {
        text: "Filters",
        description: "Button that shows or hides the extra filters row.",
    },
    "music.filter.hideInstrumentals": {
        text: "Hide instrumentals",
        description: "Switch that hides instrumental tracks from song lists.",
    },
    "music.filter.reset": {
        text: "Reset",
        description: "Button that clears the search, sort and filters.",
    },
    "music.list.trackCount": {
        text: "{count, plural, one {# track} other {# tracks}}",
        description: "A track count under an album, artist or playlist.",
    },
    "music.list.title": {
        text: "Title",
        description: "Song list column header.",
    },
    "music.list.album": {
        text: "Album",
        description: "Song list column header.",
    },
    "music.list.artist": {
        text: "Artist",
        description: "Song list column header.",
    },
    "music.list.like": {
        text: "Like",
        description: "Hidden song list column header over the heart buttons.",
    },
    "music.list.add": {
        text: "Add",
        description: "Hidden song list column header over the add buttons.",
    },
    "music.list.remove": {
        text: "Remove from playlist",
        description: "Hidden column header and tooltip of the remove buttons in a playlist.",
    },
    "music.list.empty": {
        text: "No tracks match.",
        description: "Shown in place of a song list that is empty.",
    },
    "music.list.noAlbums": {
        text: "No albums match.",
        description: "Shown in place of the album grid when the search leaves nothing.",
    },
    "music.list.noArtists": {
        text: "No artists match.",
        description: "Shown in place of the artist list when the search leaves nothing.",
    },
    "music.list.inst": {
        text: "INST",
        description: "Short pill on an instrumental track. Keep it very short.",
    },
    "music.list.hasLyrics": {
        text: "Has lyrics",
        description: "Tooltip of the small mic icon on a track with lyrics.",
    },
    "music.list.likeTrack": {
        text: "Like {name}",
        description: "Accessible name of the heart button on a track.",
    },
    "music.list.unlike": {
        text: "Unlike {name}",
        description: "Accessible name of the heart button on a liked track.",
    },
    "music.list.addTrack": {
        text: "Queue and playlist options for {name}",
        description: "Accessible name of the add button on a track.",
    },
    "music.list.addTitle": {
        text: "Queue and playlist options",
        description: "Tooltip of the add button on a track.",
    },
    "music.list.removeTrack": {
        text: "Remove {name} from this playlist",
        description: "Accessible name of the remove button on a playlist track.",
    },
    "music.list.select": {
        text: "Select {name}",
        description: "Accessible name of a track's checkbox.",
    },
    "music.list.selectAll": {
        text: "Select all shown",
        description: "Accessible name of the checkbox in the song list header.",
    },
    "music.list.selected": {
        text: "{count, plural, one {# selected} other {# selected}}",
        description: "Count in the bar that appears when tracks are selected.",
    },
    "music.list.actions": {
        text: "Actions",
        description: "Button in the selection bar that opens the queue and playlist menu.",
    },
    "music.list.actionsTitle": {
        text: "Queue and playlist options for the selection",
        description: "Tooltip of the Actions button.",
    },
    "music.list.clear": {
        text: "Clear",
        description: "Button in the selection bar that unselects everything.",
    },
    "music.list.reorder": {
        text: "Drag to move {name}",
        description: "Accessible name of a playlist row's drag handle.",
    },
    "music.list.reorderHint": {
        text: "Drag to reorder, or Alt+↑/↓ on a row",
        description: "Tooltip of a playlist row's drag handle.",
    },
    "music.list.missingLiked": {
        text: "{count, plural, one {# liked track isn't} other {# liked tracks aren't}} in the catalogue right now.",
        description: "Note over Liked Songs when some liked tracks are unknown to the current catalogue.",
    },
    "music.list.missing": {
        text: "{count, plural, one {# track isn't} other {# tracks aren't}} in the catalogue right now.",
        description: "Note over a playlist or Recently Played when some tracks are unknown to the current catalogue.",
    },
    "music.menu.playNext": {
        text: "Play next",
        description: "Menu item that puts tracks right after the current one.",
    },
    "music.menu.addToQueue": {
        text: "Add to queue",
        description: "Menu item that puts tracks at the end of the queue.",
    },
    "music.menu.all": {
        text: "{label} (all {count})",
        description: "A menu item acting on several tracks. {label} is the item, {count} the number of tracks.",
    },
    "music.menu.addTo": {
        text: 'Add to "{name}"',
        description: "Menu item that adds tracks to a playlist.",
    },
    "music.menu.addToAdded": {
        text: 'Add to "{name}" (added)',
        description: "Disabled menu item for a playlist that already holds every chosen track.",
    },
    "music.menu.newPlaylist": {
        text: "New playlist…",
        description: "Menu item that creates a playlist with the chosen tracks.",
    },
    "music.menu.added": {
        text: '{count, plural, one {Added # track} other {Added # tracks}} to "{name}"',
        description: "Toast after adding tracks to a playlist.",
    },
    "music.menu.alreadyIn": {
        text: 'Already in "{name}"',
        description: "Toast when every chosen track is already in the playlist.",
    },
    "music.menu.created": {
        text: 'Made "{name}" with {count, plural, one {# track} other {# tracks}}',
        description: "Toast after creating a playlist from the menu.",
    },
    "music.playlist.new": {
        text: "New playlist",
        description: "Tooltip and accessible name of the + button by Playlists.",
    },
    "music.playlist.newTitle": {
        text: "New playlist",
        description: "Title of the dialog that names a new playlist.",
    },
    "music.playlist.renameTitle": {
        text: "Rename playlist",
        description: "Title of the dialog that renames a playlist.",
    },
    "music.playlist.rename": {
        text: "Click to rename",
        description: "Tooltip of a playlist's name.",
    },
    "music.playlist.nameLabel": {
        text: "Playlist name",
        description: "Accessible name of the playlist name field.",
    },
    "music.playlist.namePlaceholder": {
        text: "e.g. Late night farming",
        description: "Placeholder of the playlist name field.",
    },
    "music.playlist.cancel": {
        text: "Cancel",
        description: "Button that closes a playlist dialog without changes.",
    },
    "music.playlist.create": {
        text: "Create",
        description: "Button that creates the playlist.",
    },
    "music.playlist.save": {
        text: "Save",
        description: "Button that saves a playlist's new name.",
    },
    "music.playlist.createFailed": {
        text: "Couldn't make the playlist. Try again in a bit.",
        description: "Toast when creating a playlist fails or the limit is reached.",
    },
    "music.playlist.delete": {
        text: "Delete playlist",
        description: "Button that deletes the open playlist.",
    },
    "music.playlist.deleteTitle": {
        text: "Delete playlist?",
        description: "Title of the confirmation to delete a playlist.",
    },
    "music.playlist.deleteConfirm": {
        text: 'Delete playlist "{name}"? This can\'t be undone.',
        description: "Body of the confirmation to delete a playlist.",
    },
    "music.playlist.deleteAction": {
        text: "Delete",
        description: "Button that confirms deleting a playlist.",
    },
    "music.playlist.gone": {
        text: "This playlist doesn't exist anymore.",
        description: "Shown when a playlist link points at a deleted playlist.",
    },
    "music.album.kicker": {
        text: "Monster Siren Records",
        description: "Small label above an album's title.",
    },
    "music.album.subline": {
        text: "{artists} · {count, plural, one {# track} other {# tracks}}",
        description: "Line under an album's title.",
    },
    "music.album.play": {
        text: "Play",
        description: "Button that plays an album or playlist from the start.",
    },
    "music.album.pause": {
        text: "Pause",
        description: "The album's Play button while the album is playing.",
    },
    "music.album.shuffle": {
        text: "Shuffle",
        description: "Button that turns shuffle on (starting the album at a random track) or off.",
    },
    "music.album.addAll": {
        text: "Add all",
        description: "Button that opens the queue and playlist menu for every track of the album.",
    },
    "music.album.addAllTitle": {
        text: "Queue and playlist options for the whole album",
        description: "Tooltip of the Add all button.",
    },
    "music.album.copyLink": {
        text: "Copy album link",
        description: "Tooltip of the share button on an album.",
    },
    "music.album.copied": {
        text: "Link copied",
        description: "Tooltip of the share button right after copying.",
    },
    "music.album.artist": {
        text: "Artist",
        description: "Album details row label.",
    },
    "music.album.tracks": {
        text: "Tracks",
        description: "Album details row label.",
    },
    "music.album.withLyrics": {
        text: "With lyrics",
        description: "Album details row label: how many tracks have lyrics.",
    },
    "music.album.instrumentals": {
        text: "Instrumentals",
        description: "Album details row label: how many tracks are instrumental.",
    },
    "music.album.since": {
        text: "On Monster Siren since",
        description: "Album details row label: the day the release appeared on the label's site.",
    },
    "music.album.lyricsMeta": {
        text: "{count, plural, one {# with lyrics} other {# with lyrics}}",
        description: "Phone meta line on an album: how many tracks have lyrics.",
    },
    "music.album.sinceMeta": {
        text: "On Monster Siren since {date}",
        description: "Phone meta line on an album. {date} is a formatted date.",
    },
    "music.album.more": {
        text: "More",
        description: "Button that expands a clamped album intro.",
    },
    "music.album.less": {
        text: "Less",
        description: "Button that clamps an expanded album intro.",
    },
    "music.album.loading": {
        text: "Loading album…",
        description: "Shown in the album window while the catalogue loads.",
    },
    "music.album.failed": {
        text: "This album could not be loaded. Close it and open it again to try again.",
        description: "Shown in the album window when the album is not in the catalogue.",
    },
    "music.stats.played": {
        text: "Tracks played",
        description: "Stats tile label: total plays.",
    },
    "music.stats.time": {
        text: "Listening time",
        description: "Stats tile label: total time listened.",
    },
    "music.stats.since": {
        text: "Listening since",
        description: "Stats tile label: the first play's date.",
    },
    "music.stats.never": {
        text: "N/A",
        description: "Stats tile value when nothing was played yet.",
    },
    "music.stats.seconds": {
        text: "{seconds}s",
        description: "Listening time under a minute. Keep the unit short.",
    },
    "music.stats.minutes": {
        text: "{minutes}m",
        description: "Listening time under an hour. Keep the unit short.",
    },
    "music.stats.hours": {
        text: "{hours}h {minutes}m",
        description: "Listening time of an hour or more. Keep the units short.",
    },
    "music.stats.topTracks": {
        text: "Top tracks",
        description: "Stats column heading. Also the queue source name when playing from it.",
    },
    "music.stats.topArtists": {
        text: "Top artists",
        description: "Stats column heading.",
    },
    "music.stats.topAlbums": {
        text: "Top albums",
        description: "Stats column heading.",
    },
    "music.stats.times": {
        text: "{count}×",
        description: "How many times something was played, beside a top entry.",
    },
    "music.stats.nothing": {
        text: "Nothing yet.",
        description: "Shown in an empty stats column.",
    },
    "music.stats.noArtists": {
        text: "No guest artists played yet.",
        description: "Shown in the Top artists column when only the house label was played.",
    },
    "music.stats.empty": {
        text: "No listening history yet. Play something first.",
        description: "Shown in place of the stats when nothing was played.",
    },
    "music.stats.localNote": {
        text: "You're not signed in, so likes, playlists and stats live in this browser only.",
        description: "Note under the stats for a signed-out listener.",
    },
    "music.rail.charts": {
        text: "Charts",
        description: "Library rail entry for the community charts: the most played tracks and the top listeners across the site.",
    },
    "music.list.listens": {
        text: "Plays",
        description: "Column heading over each track's community play count in a song list.",
    },
    "music.counts.exact": {
        text: "{listens, plural, one {# play} other {# plays}} · {downloads, plural, one {# download} other {# downloads}}",
        description: "Tooltip on a track's compact play count: the exact community plays and downloads of all time.",
    },
    "music.download.busy": {
        text: "Downloading {name}",
        description: "Title of the toast while a track downloads. {name} is the track title.",
    },
    "music.download.starting": {
        text: "Starting",
        description: "Toast line before the first bytes of a download arrive.",
    },
    "music.download.progress": {
        text: "{percent} done",
        description: "Toast line with a download's progress. {percent} is a formatted percentage.",
    },
    "music.download.received": {
        text: "{size} MB so far",
        description: "Toast line with a download's progress when its full size is unknown. {size} is megabytes received, already formatted with one decimal.",
    },
    "music.download.saved": {
        text: "Saved {name}",
        description: "Toast after a track was written to the file the listener picked. {name} is the track title.",
    },
    "music.download.failedTrack": {
        text: "Couldn't download {name}. Try again in a bit.",
        description: "Error toast when a track download failed. {name} is the track title.",
    },
    "music.menu.download": {
        text: "Download",
        description: "Track menu item that saves the track's audio file.",
    },
    "music.menu.downloading": {
        text: "Downloading",
        description: "Track menu item, disabled, while the track downloads and its size is unknown.",
    },
    "music.menu.downloadingPercent": {
        text: "Downloading {percent}",
        description: "Track menu item, disabled, while the track downloads. {percent} is a formatted percentage.",
    },
    "music.menu.listeners": {
        text: "Top listeners",
        description: "Track menu item that opens the signed-in listeners who played the track most.",
    },
    "music.charts.window.week": {
        text: "This week",
        description: "Charts span button: the last 7 days.",
    },
    "music.charts.window.month": {
        text: "This month",
        description: "Charts span button: the last 30 days.",
    },
    "music.charts.window.all": {
        text: "All time",
        description: "Charts span button: every play ever counted.",
    },
    "music.charts.windowAria": {
        text: "Time span",
        description: "Accessible name of the buttons that pick the span a chart covers.",
    },
    "music.charts.sort.listens": {
        text: "Plays",
        description: "Button that ranks listeners by how many plays counted.",
    },
    "music.charts.sort.time": {
        text: "Time",
        description: "Button that ranks listeners by how long they listened.",
    },
    "music.charts.sortAria": {
        text: "Rank listeners by",
        description: "Accessible name of the buttons that pick what the listener ranking orders by.",
    },
    "music.charts.topTracks": {
        text: "Top tracks",
        description: "Charts column heading: the most played tracks across the site. Also the queue source name when playing from it.",
    },
    "music.charts.topListeners": {
        text: "Top listeners",
        description: "Charts column heading: the signed-in listeners who played the most.",
    },
    "music.charts.listens": {
        text: "{count, plural, one {# play} other {# plays}}",
        description: "How many plays counted for a track or a listener.",
    },
    "music.charts.level": {
        text: "Lv. {level}",
        description: "A listener's game account level beside their name. Keep it short.",
    },
    "music.charts.profile": {
        text: "{rank}. {name}, {listens}, {time}. Open profile",
        description: "Accessible name of a ranked listener row, which links to the profile. {listens} is a play count, {time} a listening time.",
    },
    "music.charts.trackAria": {
        text: "{rank}. Play {name}, {listens}",
        description: "Accessible name of a top track row, which plays the chart from that track. {listens} is a play count.",
    },
    "music.charts.trackListeners": {
        text: "Top listeners",
        description: "Tooltip of the button on a top track row that opens the track's top listeners.",
    },
    "music.charts.trackListenersOf": {
        text: "Top listeners of {name}",
        description: "Accessible name of that button. {name} is the track title.",
    },
    "music.charts.trackKicker": {
        text: "Top listeners",
        description: "Small label above the track title in the dialog listing one track's top listeners.",
    },
    "music.charts.trackTotals": {
        text: "{listens, plural, one {# play} other {# plays}} and {downloads, plural, one {# download} other {# downloads}} of all time",
        description: "Line under the track title in the top listeners dialog: the track's community totals.",
    },
    "music.charts.failed": {
        text: "Couldn't load the charts. Try again in a bit.",
        description: "Shown in place of a ranking when its request failed.",
    },
    "music.charts.unavailable": {
        text: "Charts aren't available yet.",
        description: "Shown in place of a ranking when the backend has no charts (it predates them).",
    },
    "music.charts.noTracks": {
        text: "Nothing has been played in this span yet.",
        description: "Shown in place of the top tracks when the span has no counted plays.",
    },
    "music.charts.nobody": {
        text: "No signed-in listener in this span yet.",
        description: "Shown in place of the top listeners when nobody ranks in the span.",
    },
    "music.charts.trackNobody": {
        text: "No signed-in listener has played this in that span yet.",
        description: "Shown in the top listeners dialog when nobody ranks for the track in the span.",
    },
    "music.charts.more": {
        text: "Show more",
        description: "Button under a ranking that loads more rows.",
    },
    "music.charts.ranked": {
        text: "{count, plural, one {# listener ranked} other {# listeners ranked}}",
        description: "Under the top listeners: how many listeners the ranking covers in the span.",
    },
    "music.charts.publicNote": {
        text: "Only signed-in listeners with a public profile are ranked.",
        description: "Note under a listener ranking.",
    },
    "music.charts.rules": {
        text: "A play counts after 30 seconds, or half of a shorter track, at most once per listener and track every 30 minutes. A download counts at most once a day.",
        description: "Note under the charts explaining when a play or a download counts.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
