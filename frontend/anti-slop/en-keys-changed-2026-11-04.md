# EN message keys changed by the copy pass, 2026-11-04

For the other locales to follow. Format: namespace file, key, old -> new. Keys were not renamed; new keys added by fix waves are not listed here (run i18n:check for missing translations).

| file | key | old | new |
|---|---|---|---|
| admin/sections/Home/Home.messages.ts | home.access.note.superAdmin | Change anyone’s role and grants from People. | Change anyone's role and grants from People. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.access.changeLevel | Change {name}’s level on {title} | Change {name}'s level on {title} |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.access.loadError | Couldn’t load the grants. | Couldn't load the grants. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.access.toast.grantFailed | Couldn’t create the grant | Couldn't create the grant |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.access.toast.levelFailed | Couldn’t change the level | Couldn't change the level |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.access.toast.revokeFailed | Couldn’t revoke access | Couldn't revoke access |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.delete.body | The list, its tiers and every published version are removed for everyone. This can’t be undone. | The list, its tiers and every published version are removed for everyone. This can't be undone. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.delete.failed | Couldn’t delete the list | Couldn't delete the list |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.flair.desc | The flair shows as a coloured tag on {title} across the site. | The flair shows as a colored tag on {title} across the site. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.flair.failed | Couldn’t change the flair | Couldn't change the flair |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.head.subOwn | The tier lists you’ve been granted. Your level on each list decides whether you can publish. | The tier lists you've been granted. Your level on each list decides whether you can publish. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.head.subStaff | Lists marked official appear in the site’s Official rail. Publishing saves a new version that visitors see. | Lists marked official appear in the site's Official rail. Publishing saves a new version that visitors see. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.lists.empty.noAccess | You don’t have access to any tier lists yet. | You don't have access to any tier lists yet. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.lists.loadError | Couldn’t load the tier lists. Try again in a moment. | Couldn't load the tier lists. Try again in a moment. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.new.descPlaceholder | Optional. Shown under the title on the list’s page. | Optional. Shown under the title on the list's page. |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.new.failed | Couldn’t create the list | Couldn't create the list |
| admin/sections/TierLists/TierLists.messages.ts | tierLists.publish.failed | Couldn’t publish | Couldn't publish |
| announcements/Announcements.messages.ts | error.body | Something went wrong reaching the server. Give it another try. | The server didn't answer. Try again in a moment. |
| archives/Archives.messages.ts | file.examMissing | {count, plural, one {# of the six ratings isn't on file, so that corner sits at the centre.} other {# of the six ratings aren't on file, so those corners sit at the centre.}} | {count, plural, one {# of the six ratings isn't on file, so that corner sits at the center.} other {# of the six ratings aren't on file, so those corners sit at the center.}} |
| archives/Archives.messages.ts | numbers.infectionByNote | The gap between the two colours is files that say something neither reading can settle. Groups of fewer than {threshold} are left out. | The gap between the two colors is files that say something neither reading can settle. Groups of fewer than {threshold} are left out. |
| changelog/ChangelogError.messages.ts | error.body | We couldn't reach GitHub to fetch the latest commits. This is usually a temporary network hiccup or an API rate limit - try again in a moment. | We couldn't reach GitHub to fetch the latest commits. This is usually a temporary network hiccup or an API rate limit, so try again in a moment. |
| changelog/ChangelogError.messages.ts | error.home | Return home | Go to the home page |
| changelog/ChangelogError.messages.ts | error.unexpected | An unexpected error occurred. | No details were given. |
| changelog/ChangelogPage.messages.ts | page.truncated | Older history is capped - see the full log on GitHub. | Older history is capped. See the full log on GitHub. |
| changelog/ChangelogPage.messages.ts | range.month.label | This Month | This month |
| changelog/ChangelogPage.messages.ts | range.quarter.label | 3 Months | 3 months |
| changelog/ChangelogPage.messages.ts | range.week.label | This Week | This week |
| enemies/detail/Enemies.messages.ts | detail.notFound.back | Back to Enemy Database | Back to enemies |
| enemies/detail/impl/AppearsInTab.messages.ts | appears.category.modes | Permanent Game Modes | Permanent game modes |
| enemies/detail/impl/AppearsInTab.messages.ts | appears.category.stages | Story Stages | Story stages |
| enemies/detail/impl/AppearsInTab.messages.ts | appears.empty.body | This enemy isn't listed in any of the currently extracted level files. Coverage grows as more content is processed. | This enemy isn't in any stage we have data for yet. |
| enemies/detail/impl/sections.messages.ts | stats.forms.note | This enemy transforms across {forms} forms in-fight, but Hypergryph's data ships {count, plural, one {a single stat block} other {# stat blocks}}. Subsequent forms modify behavior through skill triggers rather than swapping the base HP/ATK/DEF - see {traits} for the form descriptions. | This enemy transforms across {forms} forms in-fight, but Hypergryph's data ships {count, plural, one {a single stat block} other {# stat blocks}}. Subsequent forms modify behavior through skill triggers rather than swapping the base HP/ATK/DEF; see {traits} for the form descriptions. |
| enemies/list/Enemies.messages.ts | list.blurb | View all {count} enemies catalogued in Arknights, sorted by Hypergryph's internal sort index by default. | View all {count} enemies cataloged in Arknights, sorted by Hypergryph's internal sort index by default. |
| enemies/list/Enemies.messages.ts | list.empty.body | Adjust the chip rail or your search above, or clear everything and start fresh. | Change the filters or the search above, or clear them all. |
| enemies/list/Enemies.messages.ts | list.empty.clear | Clear all filters | Clear filters |
| enemies/list/impl/EnemyChibi.messages.ts | chibi.error | Failed to load chibi data. | Couldn't load the chibi. |
| gacha/community/impl/PageHeader.messages.ts | community.header.title | What everyone’s {emphasis}. | What everyone's {emphasis}. |
| gacha/community/impl/RarityPanel.messages.ts | community.rarity.legend.note | expected rates fold 6★ soft pity into the advertised 2/8/50/40% base - long-run 6★ is ~2.89%, and lower rarities dip slightly because a pity 6★ takes their slot | expected rates fold 6★ soft pity into the advertised 2/8/50/40% base: long-run 6★ is ~2.89%, and lower rarities dip slightly because a pity 6★ takes their slot |
| gacha/history/HistoryPage.messages.ts | history.error.load | Couldn’t load pull records. | Couldn't load pull records. |
| gacha/history/HistoryPage.messages.ts | history.error.unknown | Unknown error. | No details were given. |
| gacha/history/HistoryPage.messages.ts | history.refresh.errBody | Try signing in again. | Your records didn't refresh. Try again in a moment. |
| gacha/history/HistoryPage.messages.ts | history.refresh.storageOff | “Store gacha history” is off, so new pulls aren’t synced. Pulls already saved stay here. | “Store gacha history” is off, so new pulls aren't synced. Pulls already saved stay here. |
| gacha/history/impl/ShareHistory.messages.ts | history.share.profilePrivate.action | Open Settings | Open settings |
| gallery/music/MusicGallery.messages.ts | music.album.failed | This album could not be loaded. Close it and open it again to try again. | Couldn't load this album. Close it and open it again. |
| gallery/music/MusicGallery.messages.ts | music.charts.failed | Couldn't load the charts. Try again in a bit. | Couldn't load the charts. Try again in a moment. |
| gallery/music/MusicGallery.messages.ts | music.counts.noSiren | The server doesn't have the Monster Siren Records catalogue yet, so only the game's {tracks, plural, one {# track} other {# tracks}} are here. | The server doesn't have the Monster Siren Records catalog yet, so only the game's {tracks, plural, one {# track} other {# tracks}} are here. |
| gallery/music/MusicGallery.messages.ts | music.counts.syncing | Syncing the label catalogue… This takes a minute the first time the server starts. | Loading the label catalog… The first load can take a minute. |
| gallery/music/MusicGallery.messages.ts | music.download.failedTrack | Couldn't download {name}. Try again in a bit. | Couldn't download {name}. Try again in a moment. |
| gallery/music/MusicGallery.messages.ts | music.failed | Couldn't load the music list. Try again in a bit. | Couldn't load the music list. Try again in a moment. |
| gallery/music/MusicGallery.messages.ts | music.list.missing | {count, plural, one {# track isn't} other {# tracks aren't}} in the catalogue right now. | {count, plural, one {# track isn't} other {# tracks aren't}} in the catalog right now. |
| gallery/music/MusicGallery.messages.ts | music.list.missingLiked | {count, plural, one {# liked track isn't} other {# liked tracks aren't}} in the catalogue right now. | {count, plural, one {# liked track isn't} other {# liked tracks aren't}} in the catalog right now. |
| gallery/music/MusicGallery.messages.ts | music.playlist.createFailed | Couldn't make the playlist. Try again in a bit. | Couldn't make the playlist. Try again in a moment. |
| gallery/music/MusicGallery.messages.ts | music.rail.history | Recently Played | Recently played |
| gallery/music/MusicGallery.messages.ts | music.rail.liked | Liked Songs | Liked songs |
| gallery/music/MusicGallery.messages.ts | music.rail.songs | All Songs | All songs |
| gallery/music/MusicGallery.messages.ts | music.rail.stats | Your Stats | Your stats |
| gallery/music/MusicGallery.messages.ts | music.siren.failed | Couldn't load the Monster Siren Records catalogue. Try again in a bit. | Couldn't load the Monster Siren Records catalog. Try again in a moment. |
| gallery/music/MusicGallery.messages.ts | music.siren.missing | The server doesn't have the Monster Siren Records catalogue yet. | The server doesn't have the Monster Siren Records catalog yet. |
| grids/EntityPickerDialog.messages.ts | picker.error | This list could not be loaded. | Couldn't load this list. |
| grids/EntityPickerDialog.messages.ts | picker.retry | Retry | Try again |
| grids/GridBoard.messages.ts | cell.addItem | Add Item | Add item |
| grids/GridEditor.messages.ts | edit.toast.saveFailedTitle | Could not save | Couldn't save the grid |
| grids/GridView.messages.ts | view.copy.errTitle | Could not copy the link | Couldn't copy the link |
| grids/GridView.messages.ts | view.fork.errTitle | Could not copy this grid | Couldn't copy this grid |
| grids/GridsBrowse.messages.ts | browse.error | Grids could not be loaded. | Couldn't load grids. |
| grids/MyGrids.messages.ts | my.error | Your grids could not be loaded. | Couldn't load your grids. |
| header/Header.messages.ts | header.collection.archives.desc | Personnel files, the faction network and the roster counted up. | Personnel files, the faction network and the roster counted up |
| header/Header.messages.ts | header.collection.enemies.desc | Every enemy catalogued, with stats and traits. | Every enemy cataloged, with stats and traits |
| header/Header.messages.ts | header.collection.music.desc | Every track in the game, where it plays, and a player that loops it like the game does. | Every track in the game, where it plays, and a player that loops it like the game does |
| header/Header.messages.ts | header.collection.operators.desc | Every operator released in Arknights. | Every operator released in Arknights |
| header/Header.messages.ts | header.collection.stages.desc | Every stage, mapped with an enemy-pathing simulator. | Every stage, mapped with an enemy-pathing simulator |
| header/Header.messages.ts | header.collection.stories.desc | Read every story in the game, with your progress synced. | Read every story in the game, with your progress synced |
| header/impl/AuthDialog.messages.ts | authDialog.biliSmsExperimental | Experimental: this endpoint hasn't been confirmed against a real response, so it may fail even with a correct code. | Experimental: Bilibili SMS sign-in hasn't been tested with a real account, so it may fail even with a correct code. |
| header/impl/AuthDialog.messages.ts | authDialog.description.bili | Log in with your Bilibili account. Your password is used once to sign in and never stored. | Sign in with your Bilibili account. Your password is used once to sign in and never stored. |
| header/impl/AuthDialog.messages.ts | authDialog.description.cn | Experimental: logs in with your Hypergryph (CN) account through the same passport flow the Skland app uses. Untested against a real account, so it may fail even with correct credentials. Your password or code is used once to sign in and never stored. | Experimental: signs in with your Hypergryph (CN) account through the same passport flow the Skland app uses. Untested against a real account, so it may fail even with correct credentials. Your password or code is used once to sign in and never stored. |
| header/impl/AuthDialog.messages.ts | authDialog.loggingIn | Logging in... | Signing in… |
| header/impl/AuthDialog.messages.ts | authDialog.login | Login | Sign in |
| header/impl/AuthDialog.messages.ts | authDialog.sendCode | Send Code | Send code |
| header/impl/AuthDialog.messages.ts | authDialog.sending | Sending... | Sending… |
| header/impl/AuthDialog.messages.ts | authDialog.title | Login | Sign in |
| header/impl/AuthDialog.messages.ts | authDialog.toast.codeError | There was an error sending a code:\n{error} | {error} |
| header/impl/AuthDialog.messages.ts | authDialog.toast.error | Error | Couldn't send the code |
| header/impl/AuthDialog.messages.ts | authDialog.toast.loggedIn | Logged in successfully. | Signed in |
| header/impl/AuthDialog.messages.ts | authDialog.toast.loginFailed | Login failed | Couldn't sign in |
| header/impl/AuthDialog.messages.ts | authDialog.toast.otpError | There was an error sending an OTP:\n{error} | {error} |
| header/impl/AuthDialog.messages.ts | authDialog.toast.sentCode | Sent code | Code sent |
| header/impl/AuthDialog.messages.ts | authDialog.toast.sentOtpEmail | Sent OTP code to your email. | Check your email for the code. |
| header/impl/AuthDialog.messages.ts | authDialog.toast.sentSms | Sent SMS code to your phone. | Check your phone for the code. |
| header/impl/MobileNav.messages.ts | mobileNav.login | Login | Sign in |
| header/impl/MobileNav.messages.ts | mobileNav.logout | Logout | Sign out |
| header/impl/UserMenu.messages.ts | userMenu.adminPanel | Admin Panel | Admin |
| header/impl/UserMenu.messages.ts | userMenu.login | Login | Sign in |
| header/impl/UserMenu.messages.ts | userMenu.logout | Logout | Sign out |
| operators/detail/impl/CommunitySharePill.messages.ts | share.tooltip | {users} of {total} {cohort}{top, select, yes { - the most common choice} other {}} | {users} of {total} {cohort}{top, select, yes {, the most common choice} other {}} |
| operators/detail/impl/LevelBreakdown.messages.ts | breakdown.rowTooltip | {users} of {total} {cohort} ({pct}%){own, select, yes { - where you are} other {}} | {users} of {total} {cohort} ({pct}%){own, select, yes {, where you are} other {}} |
| operators/detail/impl/OperatorNotes.messages.ts | notes.title | Operator Notes | Operator notes |
| operators/detail/impl/chibi/ChibiViewer.messages.ts | chibi.error.failed | Failed to load chibi | Couldn't load the chibi |
| operators/detail/impl/chibi/ChibiViewer.messages.ts | chibi.error.noData | No spine data available | No chibi for this operator |
| operators/detail/impl/chibi/ChibiViewer.messages.ts | chibi.title | Chibi Preview | Chibi preview |
| operators/detail/impl/chibi/download-button.messages.ts | chibiExport.frameRate | Frame Rate | Frame rate |
| operators/detail/impl/chibi/stage-tools.messages.ts | chibiStage.bg.custom | Custom colour | Custom color |
| operators/detail/impl/dynillust/SceneIllust.messages.ts | scene.error | Failed to load animation | Couldn't load the animation |
| operators/detail/impl/tabs/AudioContent.messages.ts | audio.search.sfx.placeholder | Search sound effects... | Search sound effects… |
| operators/detail/impl/tabs/AudioContent.messages.ts | audio.search.voice.placeholder | Search voice lines... | Search voice lines… |
| operators/detail/impl/tabs/AudioContent.messages.ts | audio.sfx.battleVoice | Battle Voice | Battle voice |
| operators/detail/impl/tabs/AudioContent.messages.ts | audio.sfx.skillSlotVoice | Skill {slot} Voice | Skill {slot} voice |
| operators/detail/impl/tabs/AudioContent.messages.ts | audio.tab.voice | Voice Lines | Voice lines |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.combatStats | Combat Stats | Combat stats |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.controls | Operator Controls | Operator controls |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.module.none | No Module | No module |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.moduleDetails | Module Details | Module details |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.moduleDetails.levelStats | Level {level} Stats | Level {level} stats |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.talentChanges | Talent Changes | Talent changes |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.talents.unnamed | Unnamed Talent | Unnamed talent |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.title | Operator Information | Operator information |
| operators/detail/impl/tabs/InfoContent.messages.ts | info.traitChanges | Trait Changes | Trait changes |
| operators/detail/impl/tabs/LevelUpContent.messages.ts | levelup.section.operatorLevel | Operator Level | Operator level |
| operators/detail/impl/tabs/LevelUpContent.messages.ts | levelup.section.skillLevels | Skill Levels | Skill levels |
| operators/detail/impl/tabs/LevelUpContent.messages.ts | levelup.section.total | Grand Total | Grand total |
| operators/detail/impl/tabs/LoreContent.messages.ts | lore.archives | Archive Files | Archive files |
| operators/detail/impl/tabs/LoreContent.messages.ts | lore.empty.title | No Records Available | No records |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.column.description | Description / Changes | Description / changes |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.compare.selectLevels | Select Levels | Select levels |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.compare.showDiff | Show Diff | Show diff |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.compare.showFull | Show Full | Show full |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.compare.toggle | Compare Skill Levels | Compare skill levels |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.empty.body | This operator has no active skills. They may rely solely on their talent or passive abilities. | This operator has no active skills. |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.empty.title | No Skills | No skills |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.level | Skill Level | Skill level |
| operators/detail/impl/tabs/SkillsContent.messages.ts | skills.range.hideDiff | Hide Diff | Hide diff |
| operators/detail/impl/constants.messages.ts | tab.levelup | Level-Up Cost | Level-up cost |
| operators/list/Operators.messages.ts | list.clearFilters | Clear all filters | Clear filters |
| operators/list/Operators.messages.ts | list.search.placeholder | Search operators... | Search operators… |
| operators/list/impl/OperatorFilterFields.messages.ts | filters.voiceActor | Voice Actor | Voice actor |
| settings/AppearancePanel.messages.ts | appearance.cnUpdates.title | CN Updates | CN updates |
| settings/AppearancePanel.messages.ts | appearance.dataSaver.rowTitle | Data Saver Mode | Data saver mode |
| settings/AppearancePanel.messages.ts | appearance.dataSaver.title | Data Saver | Data saver |
| settings/AppearancePanel.messages.ts | appearance.topBar.title | Top Bar | Top bar |
| settings/DataPanel.messages.ts | data.yourData.desc | Everything we store is visible on your profile page. For a portable copy or a GDPR request, email privacy directly. | Everything we store is visible on your profile page. For a portable copy or a GDPR request, email privacy@myrtle.moe. |
| settings/PrivacyPanel.messages.ts | privacy.gacha.desc | Saves your synced pulls so you can browse them in Gacha → History and contribute aggregate stats. Backed by {column}. | Saves your synced pulls so you can browse them in Gacha → History and contribute aggregate stats. |
| settings/PrivacyPanel.messages.ts | privacy.leaderboards.desc | Opt in to ranked appearance on {path}. Scores are still calculated either way. Backed by {column}. | Lists you on {path}. Your score is calculated either way. |
| settings/PrivacyPanel.messages.ts | privacy.publicGacha.desc | Shows your stored pulls on your profile's Pulls tab and through the links you share from Gacha → History. Off by default, and only on a public profile. Backed by {column}. | Shows your stored pulls on your profile's Pulls tab and through the links you share from Gacha → History. Off by default, and only on a public profile. |
| settings/PrivacyPanel.messages.ts | privacy.visibility.desc | Who can see your roster and account stats. Backed by {column}. | Who can see your roster and account stats. |
| stages/detail/StageDetail.messages.ts | detail.notFound.back | Back to Stages | Back to stages |
| stages/detail/impl/MapSettings.messages.ts | mapSettings.showEnemyIcons | Enemy Icons | Enemy icons |
| stages/detail/impl/MapSettings.messages.ts | mapSettings.showMovement | Enemy Movement | Enemy movement |
| stages/detail/impl/MapSettings.messages.ts | mapSettings.showRoutes | Route Lines | Route lines |
| stages/detail/impl/MapSettings.messages.ts | mapSettings.showTimers | Wait Timers | Wait timers |
| stages/detail/impl/MapSettings.messages.ts | mapSettings.walkingChibis | Walking Chibis | Walking chibis |
| stages/detail/impl/OverviewSection.messages.ts | overview.dpTick.info | How often you passively gain 1 DP - a lower number means DP regenerates faster. | How often you passively gain 1 DP. A lower number means DP regenerates faster. |
| stages/detail/impl/PropertiesSection.messages.ts | props.autoDeploy | Auto-Deploy (Multi) | Auto-deploy (multi) |
| stages/detail/impl/PropertiesSection.messages.ts | props.battleReplay | Battle Replay | Battle replay |
| stages/detail/impl/PropertiesSection.messages.ts | props.canPractice | Can Practice | Can practice |
| stages/detail/impl/PropertiesSection.messages.ts | props.cardsSelectable | Cards Selectable | Cards selectable |
| stages/detail/impl/PropertiesSection.messages.ts | props.predefinedSquad | Predefined Squad | Predefined squad |
| stages/detail/impl/PropertiesSection.messages.ts | props.steeringEnabled | Steering Enabled | Steering enabled |
| stages/detail/impl/PropertiesSection.messages.ts | props.storyOnly | Story Only | Story only |
| stages/detail/impl/PropertiesSection.messages.ts | props.trainingLevel | Training Level | Training level |
| stages/detail/impl/SpawnSchedule.messages.ts | spawns.hiddenRoutes | Hidden Routes | Hidden routes |
| stages/detail/impl/SpawnSchedule.messages.ts | spawns.title | Spawn Schedule | Spawn schedule |
| stages/list/StageList.messages.ts | list.blurb | Browse by story arc and code - each stage opens to a faux-3D board and enemy-pathing simulator. Ordered by episode and code. | Browse by story arc and code. Each stage opens to a faux-3D board and enemy-pathing simulator. Ordered by episode and code. |
| stages/list/StageList.messages.ts | list.title | Every operation, catalogued. | Every operation, cataloged. |
| stats/StatsPage.messages.ts | page.catalog.blurb | A faithful mirror of Hypergryph’s gamedata, recomputed on every build. | Counted from Hypergryph's game data each time the site updates. |
| stats/StatsPage.messages.ts | page.community.title | The community’s working notes. | The community's working notes. |
| stats/StatsPage.messages.ts | page.error.load | Couldn’t load site stats. | Couldn't load site stats. |
| stats/StatsPage.messages.ts | page.error.unknown | Unknown error. | No details were given. |
| stats/StatsPage.messages.ts | page.footer | Looking for gacha rates, most-pulled operators, or pull timing? Those live on {link}. This page is the meta-count - just the shape of what we host. | Looking for gacha rates, most-pulled operators, or pull timing? Those live on {link}. This page is the meta-count: just the shape of what we host. |
| story/export/export.messages.ts | export.cancelled | Export cancelled. | Export canceled. |
| story/library/StoryLibrary.messages.ts | library.tab.reading | Reading Order | Reading order |
| story/library/impl/Browse.messages.ts | browse.fallback | The running backend sends no storyline shelves yet, so the page is grouped by chapter order and release year instead of by the game's own themes. A backend restart fills them in. | Storyline groups aren't available right now, so stories are grouped by chapter order and release year instead of the game's own themes. |
| story/library/impl/CommunityTab.messages.ts | community.empty.body | This server has not computed what the community has read. It appears here once it does. | Community reading counts haven't been calculated yet. Check back later. |
| story/library/impl/IllustrationsTab.messages.ts | illustrations.item.notExtracted | Not extracted | Not available yet |
| story/library/impl/IllustrationsTab.messages.ts | illustrations.panel.missing | {count} not extracted | {count} not available yet |
| story/library/impl/IllustrationsTab.messages.ts | illustrations.panel.unavailable | This backend does not serve artwork yet. | Artwork for this chapter isn't available yet. |
| story/library/impl/IllustrationsTab.messages.ts | illustrations.panel.unavailableNote | The endpoint is GET /story/group/<id>/illustrations. Everything else on this page keeps working. | Everything else on this page still works. |
| story/library/impl/IllustrationsTab.messages.ts | illustrations.totalUnknown | Counts not on the wire yet | Counts not available yet |
| story/library/impl/OperatorDialog.messages.ts | operators.dialog.modules.emptyNote | This operator's modules carry no story text on the wire, or they have none yet. | This operator's modules have no story text yet. |
| story/library/impl/OperatorDialog.messages.ts | operators.dialog.tab.voices | Voice Lines | Voice lines |
| story/library/impl/ProgressTab.messages.ts | progress.sync.retry | Retry | Try again |
| story/library/impl/ReadingOrderTab.messages.ts | reading.row.noneExtracted | {count} not extracted | {count} not available yet |
| story/library/impl/ReadingOrderTab.messages.ts | reading.timeline.body | A timeline order says when each story happens in Terra, and no field in the game's data says that. The reference site's timeline is written by hand, story by story, so this site would be copying a judgement rather than deriving one, and a wrong guess reads as a spoiler. Release order and storyline order below are both derived from the data and are shown instead. | A timeline order says when each story happens in Terra, and no field in the game's data says that. The reference site's timeline is written by hand, story by story, so this site would be copying a judgment rather than deriving one, and a wrong guess reads as a spoiler. Release order and storyline order below are both derived from the data and are shown instead. |
| story/library/impl/StoriesTab.messages.ts | stories.card.noneExtracted | {count} not extracted | {count} not available yet |
| story/library/impl/StoriesTab.messages.ts | stories.panel.notExtracted | Not extracted | Not available yet |
| story/reader/reader.messages.ts | reader.error.noScript | This story has no script file in the served data. | This story's script isn't available yet. |
| story/reader/reader.messages.ts | settings.boxPreset.bottomCentre | Bottom centre | Bottom center |
| story/reader/reader.messages.ts | settings.boxPreset.centre | Centre | Center |
| story/reader/reader.messages.ts | settings.boxPreset.topCentre | Top centre | Top center |
| story/reader/reader.messages.ts | settings.speakerTint | Speaker colour | Speaker color |
| story/reader/reader.messages.ts | settings.speakerTintHint | The colour is sampled from the speaker's sprite on stage, and a speaker with nobody on stage keeps a hue hashed from their name. Either way it is kept readable at 4.5:1 on the box, and narration keeps the text colour. | The color is sampled from the speaker's sprite on stage, and a speaker with nobody on stage keeps a hue hashed from their name. Either way it is kept readable at 4.5:1 on the box, and narration keeps the text color. |
| story/reader/reader.messages.ts | settings.textColor | Text colour | Text color |
| story/reader/reader.messages.ts | settings.textColor.custom | Custom colour | Custom color |
| story/reader/reader.messages.ts | settings.textColor.swatch | Colour {number} | Color {number} |
| story/reader/reader.messages.ts | settings.textColorHint | Every swatch clears 4.5:1 on both the dark and the light box. The speaker's name keeps its own colour. | Every swatch clears 4.5:1 on both the dark and the light box. The speaker's name keeps its own color. |
| story/sprites/CharactersTab.messages.ts | sprites.unavailableNote | The server answering this page predates it; it appears after the next backend restart. | It hasn't reached the live site yet. Check back later. |
| tier-lists/Browse.messages.ts | browse.error | Failed to load tier lists. | Couldn't load tier lists. |
| tier-lists/Browse.messages.ts | browse.retry | Retry | Try again |
| tier-lists/Browse.messages.ts | browse.retrying | Retrying… | Trying again… |
| tier-lists/FilterToolbar.messages.ts | browse.search.placeholder | Search lists... | Search lists… |
| tier-lists/Hero.messages.ts | browse.hero.blurb | {count, plural, one {list} other {lists}} from the team and the community. Sort, filter, or browse what's hot right now. | {count, plural, one {list} other {lists}} from the team and the community. Sort and filter by type, flair or author. |
| tier-lists/Hero.messages.ts | browse.hero.blurbEmpty | Curated picks from the team alongside community-built rankings. Sort, filter, or browse what's hot right now. | Official lists from the team and lists from the community. Sort and filter by type, flair or author. |
| tier-lists/Hero.messages.ts | browse.hero.title | Find the meta. Build your own. | Tier lists |
| tier-lists/detail/TierListActions.messages.ts | detail.actions.download.errBody | Something went wrong while preparing the image. Try again in a moment. | The image couldn't be drawn. Try again in a moment. |
| tier-lists/detail/TierListActions.messages.ts | detail.actions.favorite.errBody | Something went wrong. Try again in a moment. | The favorite didn't save. Try again in a moment. |
| tier-lists/edit/EditHero.messages.ts | edit.hero.autosaveRetry | Retry | Try again |
| tier-lists/edit/TierListEditor.messages.ts | edit.toast.flairFailedTitle | Flair failed | Couldn't change the flair |
| tier-lists/edit/TierListEditor.messages.ts | edit.toast.publicBody | This list appears on /tier-lists. | This list now shows in the tier list browser. |
| tier-lists/edit/TierListEditor.messages.ts | edit.toast.visibilityFailedTitle | Visibility failed | Couldn't change who can see this list |
| tier-lists/my/MyHero.messages.ts | my.hero.blurbEmpty | You haven't created any tier lists yet. Spin up your first one - drafts publish instantly to your share link. | You haven't made a tier list yet. Saved changes show on its share link right away. |
| tier-lists/my/MyListCard.messages.ts | my.card.editBoard | Edit Tierlist | Edit list |
| tier-lists/my/MyListRow.messages.ts | my.row.editBoard | Edit Tierlist | Edit list |
| tier-lists/my/MyTierLists.messages.ts | my.retry | Retry | Try again |
| tier-lists/my/MyTierLists.messages.ts | my.toast.createFailed | Couldn't create list. | Couldn't create the list |
| tier-lists/my/MyTierLists.messages.ts | my.toast.deleteFailed | Couldn't delete list. | Couldn't delete the list |
| tier-lists/my/MyTierLists.messages.ts | my.toast.updateFailed | Couldn't save changes. | Couldn't save the changes |
| tier-lists/my/MyToolbar.messages.ts | my.search.placeholder | Search your lists... | Search your lists… |
| tools/birthdays/impl/FilterSidebar.messages.ts | birthdays.sidebar.reset | Reset filters | Clear filters |
| tools/dps/DpsCalculator.messages.ts | dps.help.averageDps.desc | averages skill uptime against SP recharge - the long-run sustainable rate. | averages skill uptime against SP recharge: the long-run sustainable rate. |
| tools/dps/DpsCalculator.messages.ts | dps.note.credit | All calculations go to the credit of {link} GitHub repository. | Calculation credit: the {link} GitHub repository. |
| tools/dps/impl/ResultDetail.messages.ts | dps.detail.spNotModelled | SP on hit not modelled | SP on hit not modeled |
| tools/hps/HpsCalculator.messages.ts | hps.baseHps.burstHint | Burst healer - heals only on skill activation, so there's no off-skill (base) healing. | Burst healer: heals only on skill activation, so there's no off-skill (base) healing. |
| tools/hps/HpsCalculator.messages.ts | hps.help.averageHps.desc | averages skill uptime against SP recharge - the long-run sustainable rate. | averages skill uptime against SP recharge: the long-run sustainable rate. |
| tools/hps/HpsCalculator.messages.ts | hps.help.baseHps.desc | is always-on healing during SP recharge - it's 0 for burst healers that only heal on activation. | is always-on healing during SP recharge. It's 0 for burst healers that only heal on activation. |
| tools/hps/HpsCalculator.messages.ts | hps.note.credit | All calculations go to the credit of {link} GitHub repository. | Calculation credit: the {link} GitHub repository. |
| tools/planner/BulkPlanDialog.messages.ts | planner.bulk.status.loading | Loading... | Loading… |
| tools/planner/OperatorPlanner.messages.ts | planner.deleteFailed | Failed to delete. Please try again. | Couldn't delete. Try again. |
| tools/planner/OperatorPlanner.messages.ts | planner.group.deleteConfirm | Are you sure you want to delete group "{name}"? | Delete the group "{name}"? |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.groups.deleteConfirm | Are you sure you want to delete group "{name}"? | Delete the group "{name}"? |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.groups.newName | New group name... | New group name… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.groups.placeholder | Select plan groups... | Select plan groups… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.loadingDetails | Loading operator details... | Loading operator details… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.loadingOperators | Loading operators... | Loading operators… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.pickOperator | Please select an operator to customize targets. | Pick an operator to set targets. |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.radian | Raidian's upgrades are dependent on Integrated Strategies 6 - Sui's Garden of Grotesqueries progression and cannot be planned. | Raidian's upgrades depend on Integrated Strategies 6: Sui's Garden of Grotesqueries progression and cannot be planned. |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.saving | Saving... | Saving… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.searchOperators | Search operators by name, class, or tag... | Search operators by name, class, or tag… |
| tools/planner/OperatorPlannerDialog.messages.ts | planner.dialog.selectOperator | Select Operator | Pick an operator |
| tools/planner/RequirementsPanel.messages.ts | planner.req.craftTitle | You have {have} of {required} - the remaining {shortfall} must be crafted | You have {have} of {required}; the remaining {shortfall} must be crafted |
| tools/planner/RequirementsPanel.messages.ts | planner.req.search | Search requirements... | Search requirements… |
| tools/randomizer/impl/challenges.messages.ts | randomizer.challenge.annihilationOneOperator.desc | You must do a 1 operator relay (eg. only one operator deployed at a time). | Relay with one operator: only one operator deployed at a time. |
| tools/randomizer/impl/BriefingHero.messages.ts | randomizer.hero.intro | Roll a random stage, squad, and challenge modifier. Logged-in users can restrict the draw to operators they own and stages they've completed. | Roll a random stage, squad, and challenge modifier. Signed in, you can limit the draw to operators you own and stages you've cleared. |
| tools/randomizer/impl/EmptyState.messages.ts | randomizer.empty.hintKey | Roll Squad | Roll squad |
| tools/randomizer/impl/EmptyState.messages.ts | randomizer.empty.title | Awaiting randomizer. | Nothing rolled yet. |
| tools/randomizer/impl/OperatorFiltersPanel.messages.ts | randomizer.ops.tierList.loading | Loading tiers... | Loading tiers… |
| tools/randomizer/impl/SettingsSheet.messages.ts | randomizer.settings.desc | Constrain the randomizer - by class, rarity, owned operators, or stage availability. | Constrain the randomizer by class, rarity, owned operators, or stage availability. |
| tools/randomizer/impl/SquadSlab.messages.ts | randomizer.squad.empty | No matching operators - relax your filters. | No matching operators. Relax your filters. |
| tools/recruitment/RecruitmentCalculator.messages.ts | recruit.intro | Pick the tags shown in your recruitment screen - up to {max}. Combinations are ranked by guaranteed minimum rarity - the best worst case first. Six-star operators only appear in combinations that include "Top Operator". | Pick the tags shown in your recruitment screen, up to {max}. Combinations are ranked by guaranteed minimum rarity, best worst case first. Six-star operators only appear in combinations that include "Top Operator". |
| tools/release/impl/BannersTab.messages.ts | release.banners.empty.none | The backend returned no CN pools. | No CN banners to show yet. |
| tools/release/impl/CalendarTab.messages.ts | release.calendar.empty.desc | The backend returned no rows with an EN date. | Nothing has an EN date yet. |
| tools/release/impl/EventsTab.messages.ts | release.events.empty.none | The backend returned no CN activities. | No CN events to show yet. |
| tools/release/impl/PullsPlannerTab.messages.ts | release.pulls.modelNote | Rates come from the game's own banner data: a 2% base 6★ rate, +2% per roll after 50 without one, and the rate-up split declared for each banner type. Recurring income is modelled; one-off event Orundum is not, so add it under Extra per day. | Rates come from the game's own banner data: a 2% base 6★ rate, +2% per roll after 50 without one, and the rate-up split declared for each banner type. Recurring income is modeled; one-off event Orundum is not, so add it under Extra per day. |
| tools/release/impl/SkinPopup.messages.ts | release.popup.loadFailed | Couldn't load this outfit. | Couldn't load this skin. |
| tools/release/impl/shared.messages.ts | release.loadFailed | Failed to load. | Couldn't load this tab. |
| tools/release/impl/shared.messages.ts | release.retry | Retry | Try again |
| tools/shared/OperatorPicker.messages.ts | calc.picker.loading | {noun, select, healer {Loading healers...} other {Loading operators...}} | {noun, select, healer {Loading healers…} other {Loading operators…}} |
| tools/shared/OperatorPicker.messages.ts | calc.picker.search | {noun, select, healer {Search {count, plural, one {# healer} other {# healers}}...} other {Search {count, plural, one {# operator} other {# operators}}...}} | {noun, select, healer {Search {count, plural, one {# healer} other {# healers}}…} other {Search {count, plural, one {# operator} other {# operators}}…}} |
| tools/squad-builder/analysis/SynergyPanel.messages.ts | squad.analysis.synergy.lead.unread | This operator is not on the EN server yet, so who this talent reaches is not read. | This operator is not on the EN server yet, so this talent's targets aren't known yet. |
| tools/squad-builder/impl/OperatorPicker.messages.ts | squad.picker.loading | Loading operators... | Loading operators… |
| tools/squad-builder/impl/SeatGrid.messages.ts | squad.grid.move | Move to seat... | Move to seat… |
| tools/squad-builder/impl/ShareDialog.messages.ts | squad.share.rendering | Drawing the image... | Drawing the image… |
| tools/squad-builder/impl/StagePicker.messages.ts | squad.stagePicker.loading | Loading stages... | Loading stages… |
| ui/search-command.messages.ts | searchCommand.operatorsFailed | Failed to load operators. Try reopening the palette. | Couldn't load operators. Try reopening the palette. |
| ui/sidebar.messages.ts | sidebar.toggle | Toggle Sidebar | Toggle sidebar |
| user/leaderboard/impl/GradeBadge.messages.ts | leaderboard.grade.unknown | Not graded yet - this account has not been scored. | Not graded yet: this account has not been scored. |
| user/leaderboard/impl/Hero.messages.ts | leaderboard.hero.blurb | A composite ranking across roster depth, base efficiency, and combat scoring. Pulled from public profiles only. | A composite score across operators, base, Integrated Strategies, medals, stages and Reclamation Algorithm. Public profiles only. |
| user/leaderboard/impl/Hero.messages.ts | leaderboard.hero.notice | {lead} - the leaderboard is in active development and the ranking formula is subject to change. | {lead}: the leaderboard is in active development and the ranking formula is subject to change. |
| user/leaderboard/impl/Toolbar.messages.ts | leaderboard.toolbar.search.placeholder | Search player name or UID… | Search by nickname or UID… |
| user/profile/impl/background-editor/ArtBrowser.messages.ts | profile.background.gallery.clearFilters | Clear all | Clear filters |
| user/profile/impl/background-editor/ArtBrowser.messages.ts | profile.background.gallery.retry | Retry | Try again |
| user/profile/impl/tabs/Enemies/EnemiesTab.messages.ts | profile.enemies.level.all | All Tiers | All tiers |
| user/profile/impl/tabs/Enemies/EnemiesTab.messages.ts | profile.enemies.search.placeholder | Search enemies... | Search enemies… |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.rarity.all | All Rarities | All rarities |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.rarity.placeholder | Filter by Rarity | Filter by rarity |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.search.placeholder | Search items... | Search items… |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.sort.category | Sort by Category | Sort by category |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.sort.name | Sort by Name | Sort by name |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.sort.qty | Sort by Quantity | Sort by quantity |
| user/profile/impl/tabs/Items/ItemsTab.messages.ts | profile.items.sort.rarity | Sort by Rarity | Sort by rarity |
| user/profile/impl/tabs/Items/helpers.messages.ts | profile.items.category.module | Module Mats | Module mats |
| user/profile/impl/tabs/Optimizer/base/BasePanel.messages.ts | profile.base.wip | The base optimizer is a work in progress. If a recommendation looks wrong, please report it on Discord. | The base optimizer is a work in progress. If a recommendation looks wrong, report it on Discord. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.bench.tooltip | Spare seat: this operator fills a free seat at the lowest opportunity cost. They were not chosen for their skills - any effect that still applies is a bonus. | Spare seat: this operator fills a free seat at the lowest opportunity cost. They were not chosen for their skills; any effect that still applies is a bonus. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.disposition.capacity.hint | This skill raises the room's order capacity, not its speed - it buys longer gaps between check-ins. | This skill raises the room's order capacity, not its speed. It buys longer gaps between check-ins. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.disposition.covered.hint | A stronger skill of the same type is already active in this crew - the game only applies the most effective one, so this copy adds nothing on top. | A stronger skill of the same type is already active in this crew. The game only applies the most effective one, so this copy adds nothing on top. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.disposition.morale.hint | This skill changes morale drain or recovery - it shows up in the sustainability simulation, not in this room's efficiency. | This skill changes morale drain or recovery. It shows up in the sustainability simulation, not in this room's efficiency. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.disposition.non_production.hint | Non-production value (clues, training, HR) - counted in its own units, never folded into the efficiency number. | Non-production value (clues, training, HR), counted in its own units and never folded into the efficiency number. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.disposition.per_room.hint | Active - this skill buffs matching operators in the rooms that satisfy it, so its value is counted inside those rooms' numbers. Open those rooms to see the credit. | Active: this skill buffs matching operators in the rooms that satisfy it, so its value is counted inside those rooms' numbers. Open those rooms to see the credit. |
| user/profile/impl/tabs/Optimizer/base/board/RoomPopover.messages.ts | profile.base.room.marginalNote | Values are marginals - what this room loses if that one skill is removed. Coupled skills overlap, so they don’t sum to the room total. | Values are marginals: what this room loses if that one skill is removed. Coupled skills overlap, so they don't sum to the room total. |
| user/profile/impl/tabs/Optimizer/base/board/tile/RoomTile.messages.ts | profile.base.tile.notBuilt | Not Built | Not built |
| user/profile/impl/tabs/Optimizer/base/controls/AccountFacts.messages.ts | profile.base.facts.recruitSlots.tooltip | Recruitment slots you have purchased beyond the first - account state the sync cannot read. Declaring it prices skills like Lin’s Meritocracy (+10% HR speed per slot); left at +0 they honestly count as zero. | Recruitment slots you have purchased beyond the first. The sync cannot read this. Declaring it prices skills like Lin's Meritocracy (+10% HR speed per slot); left at +0 they honestly count as zero. |
| user/profile/impl/tabs/Optimizer/base/controls/PromotionToggle.messages.ts | profile.base.ignorePromotion.tooltip | Plan with every operator’s highest base skills, even the ones they are not promoted far enough to use. Skills you have not unlocked yet show greyed out. | Plan with every operator's highest base skills, even the ones they are not promoted far enough to use. Skills you have not unlocked yet show greyed out. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.conservative.tooltip | Order sizes are modeled conservatively (small orders, fast turnover), so real deadlines can be later than shown - never earlier. Trading posts are assumed gold-supplied. | Order sizes are modeled conservatively (small orders, fast turnover), so real deadlines can be later than shown, never earlier. Trading posts are assumed gold-supplied. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.dronesFilling | Drones {meter} - full in {when}, spend before then | Drones {meter}: full in {when}, spend before then |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.events.none | Nobody runs dry - every operator survives the recommended rotation. | Nobody runs dry: every operator survives the recommended rotation. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.furniture.line | Dormitory Lv{level} ambience {meter} - {rate} recovery sitting in the furniture shop. | Dormitory Lv{level} ambience {meter}: {rate} recovery sitting in the furniture shop. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.simulating | Simulating the rotation - this is the slow part… | Simulating the rotation. This is the slow part… |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.snapshotHint | Snapshot the current numbers, then edit the board or run the optimizer - the difference tracks live. | Snapshot the current numbers, then edit the board or run the optimizer. The difference tracks live. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.stalls | {room} stalls {duration} after a claim - buffers below. | {room} stalls {duration} after a claim. Buffers below. |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.title | Deep Dive | Deep dive |
| user/profile/impl/tabs/Optimizer/base/stats/DeepDive.messages.ts | profile.base.deep.unrotated.line | {count} if you never swap - first at {hours}h ({name}). Rotate or lose the hours. | {count} if you never swap, first at {hours}h ({name}). Rotate or lose the hours. |
| user/profile/impl/tabs/Optimizer/base/stats/StatsForNerds.messages.ts | profile.base.nerds.dormList | Dormitories - best first, game rates | Dormitories (best first, game rates) |
| user/profile/impl/tabs/Optimizer/base/stats/StatsForNerds.messages.ts | profile.base.nerds.title | Stats for Nerds | Stats for nerds |
| user/profile/impl/tabs/Optimizer/optimizers.messages.ts | profile.optimizer.account.blurb | Where to spend next across the account - promotions, masteries and modules, ranked by what each actually returns. | Where to spend next across the account: promotions, masteries and modules, ranked by what each actually returns. |
| user/profile/impl/tabs/Roster/DetailedCard.messages.ts | profile.roster.detailed.currentPotential | Current Potential | Current potential |
| user/profile/impl/tabs/Roster/OperatorDialog.messages.ts | profile.roster.dialog.combatStats | Combat Stats | Combat stats |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.search.placeholder | Search operators... | Search operators… |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.investment | Sort by Investment | Sort by investment |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.level | Sort by Level | Sort by level |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.maxed | Sort by Maxed | Sort by maxed |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.obtained | Sort by Obtained | Sort by obtained |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.potential | Sort by Potential | Sort by potential |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.rarity | Sort by Rarity | Sort by rarity |
| user/profile/impl/tabs/Roster/RosterTab.messages.ts | profile.roster.sort.trust | Sort by Trust | Sort by trust |
| user/profile/impl/tabs/Roster/UnownedCard.messages.ts | profile.roster.unowned.badge | Not Owned | Not owned |
| user/profile/impl/tabs/Score/ScoreTabEmpty.messages.ts | score.empty.desc | This player's grade hasn't been calculated. Scores are computed periodically once enough data is on file. | This player's grade hasn't been calculated yet. Grades are recalculated on a schedule, so it appears after the next run. |
| user/profile/impl/tabs/Score/cards/OverallGradeCard.messages.ts | score.ladder.top | Top of the ladder - nothing left to climb. | Top of the ladder. Nothing left to climb. |
| user/profile/impl/tabs/Score/cards/OverallGradeCard.messages.ts | score.overall.title | Overall Grade | Overall grade |
| user/profile/impl/tabs/Score/cards/ScoreHistoryCard.messages.ts | score.history.empty | History builds one point per leaderboard snapshot - check back after the next one. | History builds one point per leaderboard snapshot. Check back after the next one. |
| user/profile/impl/tabs/Score/cards/SubscoreCard.messages.ts | score.sub.shareTooltip | Worth {share}% of your overall grade - at {pct}%, this section contributes {contribution} of those {share} points | Worth {share}% of your overall grade. At {pct}%, this section contributes {contribution} of those {share} points |
| user/profile/impl/tabs/Score/improvements/MedalPanel.messages.ts | score.improvements.medal.event.title | Event medals - limited time | Event medals (limited time) |
| user/profile/impl/tabs/Score/improvements/MedalPanel.messages.ts | score.improvements.medal.hidden.tooltip | Hidden medal - unlock condition isn't shown in-game. | Hidden medal: the unlock condition isn't shown in-game. |
| user/profile/impl/tabs/Score/improvements/MedalPanel.messages.ts | score.improvements.medal.ownedTooltip | Earned by {pct}% of synced players on your server who share stats - how rare this medal is in the community. | Earned by {pct}% of synced players on your server who share stats. This is how rare the medal is in the community. |
| user/profile/impl/tabs/Score/improvements/OperatorPanel.messages.ts | score.improvements.operator.dim.rowTitleDetail | {label} - {detail}: worth {share}% of this section. You've completed {completion}% of that, earning {earned}%. | {label} ({detail}): worth {share}% of this section. You've completed {completion}% of that, earning {earned}%. |
| user/profile/impl/tabs/Score/improvements/OperatorPanel.messages.ts | score.improvements.operator.empty | Every owned operator is at their last milestone - nothing to upgrade here. | Every owned operator is at their last milestone. Nothing to upgrade here. |
| user/profile/impl/tabs/Score/improvements/OperatorPanel.messages.ts | score.improvements.operator.footnote | The header's {gain} is what finishing every upgrade here would add to your headline score. The per-tag, per-rarity, and per-operator figures below are {unit} (the percentage shown at the top of the card) - there's +{room}% of room left here to climb toward 100%. ELITE includes a full re-level at the new phase, so it overlaps MAX_LEVEL; the per-op total deduplicates the pair. | The header's {gain} is what finishing every upgrade here would add to your headline score. The per-tag, per-rarity, and per-operator figures below are {unit} (the percentage shown at the top of the card), and there's +{room}% of room left here to climb toward 100%. ELITE includes a full re-level at the new phase, so it overlaps MAX_LEVEL; the per-op total deduplicates the pair. |
| user/profile/impl/tabs/Score/improvements/OperatorPanel.messages.ts | score.improvements.operator.tag.tooltip.negligible | less than 0.01% - negligible | less than 0.01%, negligible |
| user/profile/impl/tabs/Score/improvements/RoguelikePanel.messages.ts | score.improvements.roguelike.empty | No roguelike themes loaded for this account. | No Integrated Strategies progress on this account yet. |
| user/profile/impl/tabs/Score/improvements/SandboxPanel.messages.ts | score.improvements.sandbox.empty | No RA progress detected. Play Reclamation Algorithm to begin tracking. | No Reclamation Algorithm progress synced yet. Play a run, then re-sync. |
| user/profile/impl/tabs/Score/improvements/StagePanel.messages.ts | score.improvements.stage.annihilation.lockedSubtitle | Rotating maps that aren't playable right now, so they can't be maxed until their rotation returns. These are scored on a recency curve rather than as permanent gaps - shown for reference. | Rotating maps that aren't playable right now, so they can't be maxed until their rotation returns. These are scored on a recency curve rather than as permanent gaps, and shown for reference. |
| user/profile/impl/tabs/Score/improvements/StagePanel.messages.ts | score.improvements.stage.chip.lockedTitle | {label} - rotated out, not currently playable | {label}: rotated out, not currently playable |
| user/profile/impl/tabs/Showcase/ShowcaseEditor.messages.ts | profile.showcase.add.kindTitle | Favourites of which type? | Favorites of which type? |
| user/profile/impl/tabs/Showcase/ShowcaseEditor.messages.ts | profile.showcase.editor.entitiesList | Favourites in this block | Favorites in this block |
| user/profile/impl/tabs/Showcase/ShowcaseEditor.messages.ts | profile.showcase.editor.favouritesOf | Favourite {kind} | Favorite {kind} |
| user/profile/impl/tabs/Showcase/ShowcaseEditor.messages.ts | profile.showcase.editor.type.favourites | Favourites | Favorites |
| user/profile/impl/tabs/Showcase/ShowcaseTab.messages.ts | profile.showcase.empty.desc | Pick favourite operators, skins, events or story chapters, and pin your grids, tier lists and plans. Visitors see your showcase first. | Pick favorite operators, skins, events or story chapters, and pin your grids, tier lists and plans. Visitors see your showcase first. |
| user/profile/impl/tabs/Stats/cards/ClassBreakdownCard.messages.ts | profile.stats.classes.title | Class Breakdown | Class breakdown |
| user/profile/impl/tabs/Stats/cards/CollectionCard.messages.ts | profile.stats.collection.title | Operator Collection | Operator collection |
| user/profile/impl/tabs/Stats/cards/MasteryCard.messages.ts | profile.stats.mastery.totalLevels | Total Mastery Levels | Total mastery levels |
| user/profile/impl/tabs/Stats/cards/ModulesSkinsCard.messages.ts | profile.stats.modules.unlockRate | Unlock Rate | Unlock rate |
| user/profile/impl/tabs/Stats/cards/ModulesSkinsCard.messages.ts | profile.stats.skins.collected | Skins Collected | Skins collected |
| user/profile/impl/tabs/Stats/cards/SignInCard.messages.ts | profile.stats.signin.calendar.title | Daily Sign-In | Daily sign-in |
| user/profile/impl/tabs/Stats/cards/SignInCard.messages.ts | profile.stats.signin.record.title | Sign-In Record | Sign-in record |
| user/profile/impl/tabs/Stats/cards/SkinViewerDialog.messages.ts | profile.skins.popularityFailed | Couldn't load outfit popularity, so this sort has nothing to order by. | Couldn't load skin popularity, so this sort has nothing to order by. |
| user/profile/impl/tabs/Stats/cards/SkinViewerDialog.messages.ts | profile.skins.section.all | All Skins | All skins |
| user/profile/impl/tabs/Stats/cards/SkinViewerDialog.messages.ts | profile.skins.title | Skin Collection | Skin collection |
| user/profile/impl/tabs/Stats/cards/TopOperatorsCard.messages.ts | profile.stats.top.title | Top Operators | Top operators |
| user/profile/impl/tabs/Stats/primitives.messages.ts | profile.stats.gaps.allComplete | All Complete | All complete |
| lib/meta.messages.ts | enemies.description | View every enemy catalogued in Arknights. | View every enemy cataloged in Arknights. |
| lib/meta.messages.ts | gachaCommunity.description | Community-wide gacha statistics - pull rates, most-pulled operators, and pull timing across opted-in players. | Community-wide gacha statistics: pull rates, most-pulled operators, and pull timing across opted-in players. |
| lib/meta.messages.ts | home.description | Track your roster, scout community pulls, and explore every operator. | Track your roster, scout community pulls, and look up every operator. |
| lib/meta.messages.ts | privacy.description | Your privacy matters. Here's how Myrtle protects and handles your personal information. | What Myrtle stores about you, who can see it, and how to delete it. |
| lib/meta.messages.ts | root.description | Arknights companion - operators, rosters, tier lists. | Arknights companion: operators, rosters, tier lists. |
| lib/meta.messages.ts | toolsRandomizer.description | Roll a random Arknights stage, squad, and challenge modifier. Logged-in users can constrain rolls to stages they've cleared and operators they own. | Roll a random Arknights stage, squad, and challenge modifier. Signed in, you can limit rolls to stages you've cleared and operators you own. |
| lib/meta.messages.ts | toolsRecruitment.description | Calculate optimal tag combinations for Arknights recruitment. Find guaranteed 5-star and 6-star operators with the best tag combos. | Pick your Arknights recruitment tags and see every combination, ranked by the lowest rarity it guarantees. |
| lib/og/impl/presets.messages.ts | og.enemies.subtitle | Every enemy catalogued in Arknights. | Every enemy cataloged in Arknights. |
| lib/og/impl/presets.messages.ts | og.home.subtitle | Track your roster, scout community pulls, and explore every operator. | Track your roster, scout community pulls, and look up every operator. |
| lib/og/impl/presets.messages.ts | og.login.title | Log in to Myrtle | Sign in to Myrtle |
| lib/registry/stage-groups.messages.ts | stageGroup.annihilation.desc | Endless-wave defense operations scored by total kills - the weekly source of Orundum. | Endless-wave defense operations scored by total kills, and the weekly source of Orundum. |
| lib/registry/stage-groups.messages.ts | stageGroup.cc.desc | Score-attack contracts - stack risk modifiers on a fixed map for higher rewards. | Score-attack contracts: stack risk modifiers on a fixed map for higher rewards. |
| lib/registry/stage-groups.messages.ts | stageGroup.ra.desc | A survival sandbox - scout the wilds, gather resources, build defenses and endure each cycle. | A survival sandbox: scout the wilds, gather resources, build defenses and endure each cycle. |
| lib/registry/stage-groups.messages.ts | stageGroup.story.desc | The mainline campaign - Rhodes Island's canonical operations, told episode by episode across Terra. | The mainline campaign: Rhodes Island's canonical operations, told episode by episode across Terra. |


## Second round (fc081710): error-phrasing alignment

| gacha/community/CommunityPage.messages.ts | community.error.load | The community stats couldn't be loaded. | Couldn't load the community stats. |
| gacha/community/CommunityPage.messages.ts | community.error.banners | The banner list couldn't be loaded. | Couldn't load the banner list. |
| gacha/community/CommunityPage.messages.ts | community.error.bannerStats | The per-banner pull counts couldn't be loaded, so the banners below show without them. | Couldn't load the per-banner pull counts, so the banners below show without them. |
| gacha/history/impl/HistoryBody.messages.ts | history.error.names | Operator or banner names couldn't be loaded, so some pulls show without them. | Couldn't load some operator or banner names, so those pulls show without them. |
| home/impl/HappeningNow.messages.ts | live.loading | Loading what's running on {server}. | Loading what's running on {server}… |
| home/impl/TopPlayers.messages.ts | leaders.loading | Loading the leaderboard. | Loading the leaderboard… |
| operators/detail/impl/components/OperatorNotes.messages.ts | notes.loading | Loading operator notes | Loading operator notes… |
| operators/detail/impl/components/OperatorNotes.messages.ts | notes.error | The operator notes did not load. | Couldn't load the operator notes. |
| operators/detail/impl/components/SummonsSection.messages.ts | summons.attackRange.error | The summon's attack range did not load. | Couldn't load the summon's attack range. |
| operators/detail/impl/components/tabs/AudioContent.messages.ts | audio.error.voice | The voice lines did not load. | Couldn't load the voice lines. |
| operators/detail/impl/components/tabs/InfoContent.messages.ts | info.attackRange.error | The attack range did not load. | Couldn't load the attack range. |
| operators/detail/impl/components/tabs/LevelUpContent.messages.ts | levelup.materialsFailed | Item names did not load, so tiles show item ids. | Couldn't load item names, so tiles show item ids. |
| operators/detail/impl/components/tabs/SkillsContent.messages.ts | skills.range.error | The skill's attack range did not load. | Couldn't load the skill's attack range. |
| operators/list/Operators.messages.ts | list.error.upcoming | The upcoming CN operators did not load. | Couldn't load the upcoming CN operators. |
| user/leaderboard/Leaderboard.messages.ts | leaderboard.loadFailed.players | The leaderboard did not load. | Couldn't load the leaderboard. |
| user/leaderboard/Leaderboard.messages.ts | leaderboard.loadFailed.holders | The holders of this item did not load. | Couldn't load the holders of this item. |
| user/leaderboard/Leaderboard.messages.ts | leaderboard.loadFailed.movers | The biggest movers did not load. | Couldn't load the biggest movers. |
| user/leaderboard/Leaderboard.messages.ts | leaderboard.loadFailed.items | The item list did not load. | Couldn't load the item list. |
| user/leaderboard/Leaderboard.messages.ts | leaderboard.loadFailed.standing | Your standing did not load. | Couldn't load your standing. |
| user/leaderboard/impl/components/RankByPicker.messages.ts | leaderboard.rankBy.loadFailed | The item list did not load. | Couldn't load the item list. |
| user/leaderboard/impl/components/Hero.messages.ts | leaderboard.hero.stat.failed | Did not load. Retry | Couldn't load. Try again |
| user/search/UserSearch.messages.ts | search.error.title | The search couldn't be loaded. | Couldn't search players |
| user/profile/UserProfile.messages.ts | profile.loadFailed.profile | This profile could not be loaded. The site may be having trouble; try again in a moment. | Couldn't load this profile. Try again in a moment. |
| user/profile/UserProfile.messages.ts | profile.loadFailed.roster | This player's operators could not be loaded. | Couldn't load this player's operators. |
| user/profile/UserProfile.messages.ts | profile.loadFailed.inventory | This player's items could not be loaded. | Couldn't load this player's items. |
| user/profile/UserProfile.messages.ts | profile.loadFailed.score | This player's score could not be loaded. | Couldn't load this player's score. |
| user/profile/UserProfile.messages.ts | profile.loadFailed.improvements | Upgrade suggestions could not be loaded, so the panels below show none. | Couldn't load upgrade suggestions, so the panels below show none. |
| user/profile/UserProfile.messages.ts | profile.loadFailed.enemies | This player's encountered enemies could not be loaded. | Couldn't load this player's encountered enemies. |
| user/profile/impl/components/tabs/Items/ItemsTab.messages.ts | profile.items.materialsFailed | Item names did not load, so items show their ids. | Couldn't load item names, so items show their ids. |
| user/profile/impl/components/tabs/Lookback/LookbackTab.messages.ts | lookback.error | The snapshots couldn't be loaded. | Couldn't load the snapshots. |
| user/profile/impl/components/tabs/Plans/PlansTab.messages.ts | profile.plans.loading | Loading plans | Loading plans… |
| user/profile/impl/components/tabs/Plans/PlansTab.messages.ts | profile.plans.error | The plans couldn't be loaded. | Couldn't load the plans. |
| user/profile/impl/components/tabs/Showcase/ShowcaseTab.messages.ts | profile.showcase.blockFailed.grid | This grid couldn't be loaded. | Couldn't load this grid. |
| user/profile/impl/components/tabs/Showcase/ShowcaseTab.messages.ts | profile.showcase.blockFailed.tierList | This tier list couldn't be loaded. | Couldn't load this tier list. |
| user/profile/impl/components/tabs/Showcase/ShowcaseTab.messages.ts | profile.showcase.blockFailed.plan | This plan couldn't be loaded. | Couldn't load this plan. |
| user/profile/impl/components/tabs/Stats/StatsTab.messages.ts | profile.stats.error | The statistics couldn't be loaded. | Couldn't load the statistics. |
| story/library/StoryLibrary.messages.ts | route.loadFailed | This story didn't load. Check your connection and try again. | Couldn't load this story. Check your connection and try again. |
| tier-lists/detail/TierListDetail.messages.ts | detail.snapshot.failedTitle | Version {version} didn't load | Couldn't load version {version} |
| tools/randomizer/Randomizer.messages.ts | randomizer.loadError | Some of the game data couldn't be loaded, so the operator and stage pools may be incomplete. | Couldn't load some of the game data, so the operator and stage pools may be incomplete. |
| tools/release/impl/components/PullsPlannerTab.messages.ts | release.pulls.plan.eventsError | Event names and art couldn't be loaded, so some banners show without them. | Couldn't load event names and art, so some banners show without them. |
| tools/squad-builder/impl/StagePicker.messages.ts | squad.stagePicker.error | The stage list couldn't be loaded. | Couldn't load the stage list. |
| tools/squad-builder/impl/SeatGrid.messages.ts | squad.grid.detailsFailed | Details failed to load | Couldn't load this operator's details |
| tools/dps/DpsCalculator.messages.ts | dps.import.failed | Couldn't load the operator list, so your squad wasn't added yet. | Couldn't load the operator list, so your squad wasn't added. |
| ui/route-error.messages.ts | routeError.title | Something went wrong loading this page | Couldn't load this page |
| header/impl/AuthDialog.messages.ts | authDialog.saveCredentials.hint | Keeps an encrypted game token on our server, so re-syncing works without signing in again. Untick it and nothing is saved, so a re-sync more than an hour later asks you to sign in again. Revoke it any time in Settings. | Keeps an encrypted game token on our server, so re-syncing works without signing in again. Uncheck it and nothing is saved, so a re-sync more than an hour later asks you to sign in again. Revoke it any time in Settings. |
| settings/DataPanel.messages.ts | data.linked.desc | We never store your password or sign-in code. If you ticked Keep me synced when you signed in, we keep an encrypted game token that lets us re-sync on your behalf. Disconnect below to delete it. | We never store your password or sign-in code. If you checked Keep me synced when you signed in, we keep an encrypted game token that lets us re-sync on your behalf. Disconnect below to delete it. |


Also changed in fc081710 (older keys):

| file | key text | old -> new |
|---|---|---|
| tools/dps/impl/components/ZonePanel.messages.ts | supporters error | Supporters could not be loaded: {error} -> Couldn't load the supporters: {error} |
| same | supporters paused | ...the supporter values could not be loaded ({error})... -> ...couldn't load the supporter values ({error})... |
| tier-lists/edit/KindPool.messages.ts | list error | This list did not load. -> Couldn't load this list. |
| story/library/impl/OperatorDialog.messages.ts | shelf error | This shelf did not load. ... -> Couldn't load this shelf. ... |
| story/library/impl/IllustrationsTab.messages.ts | artwork error | The artwork could not be loaded. -> Couldn't load the artwork. |
| story/sprites/CharactersTab.messages.ts | list error | The character list could not be loaded. -> Couldn't load the character list. |
| user/profile/.../Showcase/ShowcaseTab.messages.ts | showcase error | The showcase couldn't be loaded. -> Couldn't load the showcase. |
| story/export/export.messages.ts | pictures left out | {count} pictures could not be loaded and were left out. -> Couldn't load {count} pictures, so they were left out. |


## Final wave (after fee8f54a): text changes the other locales must follow

| file | key | change |
|---|---|---|
| gacha/community/impl/PageHeader.messages.ts | community.header.title | now "What everyone's pulling." with NO `{emphasis}` placeholder (the red emphasis word and `community.header.titleEmphasis` are gone): every locale must drop the placeholder |
| stages/list/StageList.messages.ts | list.counts | "{stages} stages · {zones} zones · {categories} categories" |
| stats/StatsPage.messages.ts | page.catalog.title / page.community.title | "Game catalog" / "Community tier lists" |
| story/library/StoryLibrary.messages.ts | library.counts | now includes `{stories}` ("{groups} chapters · {records} operators · {stories} stories") |
| tools/recruitment/impl/components/ResultsList.messages.ts | recruit.empty.noSelection.desc | "Combinations appear here once you pick a tag." |

Removed keys (no translation needed): stages list.title; tierLists detail.hero.kicker; stats
page.kpi.* (6), page.catalog.kicker, page.community.kicker, catalog.tierLists(.meta);
leaderboard.hero.eyebrow; community.header.titleEmphasis; story library.countsAll,
library.storiesLabel; tools randomizer.empty.kicker, squad.slots.signIn(Button),
release.planner.tag.review, release.planner.tag.listing.
