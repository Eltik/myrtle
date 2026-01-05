# Website Description

This document provides a detailed description of the myrtle.moe website, as if describing it to someone who is blind. It covers the purpose, layout, colors, components, and functionality of each page.

## General Theme and Colors

The website uses a modern, clean design with a customizable accent color. It supports both light and dark themes.

*   **Light Theme:** Features a very light gray (almost white) background with dark gray text. The default accent color is a vibrant orange-red, used for links, buttons, and highlights.
*   **Dark Theme:** Features a dark charcoal background with light gray text. The accent color is a brighter version of the one in the light theme, ensuring good contrast.

The accent color can be changed by the user, and it affects various elements across the site, including buttons, links, and decorative glows.

---

## Home Page

### Purpose

The home page serves as a landing page, introducing the website's features and providing clear calls to action for new and returning users.

### Layout

The page has a simple, single-column layout. The main content is centered in the page.

### Colors and Theme

The page uses the general theme of the website. The background is either light or dark, depending on the user's preference. The primary (accent) color is used for the looping text and highlights on the cards.

### Components

The home page is composed of two main sections:

1.  **Main Content:**
    *   A large, bold heading that reads: "Elevate your [Arknights/Operator/Farming/Recruitment] experience." The word in the brackets loops through the four options, with each word displayed in the site's accent color and a subtle glow effect.
    *   A paragraph below the heading provides a brief introduction to the website: "An advanced toolkit for the modern Doctor. Track your operators, plan your strategies, and optimize your gameplay."

2.  **Bento Grid:**
    A grid of cards below the main content, arranged in a "bento box" style layout. The grid contains the following cards:

    *   **Get Started (Large Card):**
        *   This card spans two columns in the grid.
        *   It has the title "Get Started" and a description: "Create an account to get started with your Arknights journey."
        *   It lists three numbered steps:
            1.  **Connect Your Account:** With a "Connect" button.
            2.  **View Your Profile:** With a "Profile" button.
            3.  **Star on GitHub:** With a "GitHub" button.

    *   **Statistics (Medium Card):**
        *   This card has the title "Statistics" and a description: "View our overall website statistics and data."
        *   It displays a bar chart showing website usage (Desktop vs. Mobile) over the past six months. The bars are colored in shades of the primary accent color.
        *   Below the chart, there are three key statistics in a row:
            *   **Total Visitors:** A number (e.g., 1,224).
            *   **Growth:** A percentage with a plus sign (e.g., +12%), displayed in green.
            *   **Active Users:** A number (e.g., 847).

    *   **Operator Database (Small Card):**
        *   A clickable card with the title "Operator Database" and a description: "Browse all operators with detailed stats, skills, and recommended builds."
        *   It features an icon of a group of users.
        *   When hovered over, an arrow icon on the right moves to the right, and the title moves up slightly.

    *   **Material Planner (Small Card):**
        *   A clickable card with the title "Material Planner" and a description: "Calculate resources needed for operator upgrades and mastery training."
        *   It features a target icon.
        *   It has the same hover effect as the Operator Database card.

    *   **Event Timeline (Small Card):**
        *   A clickable card with the title "Event Timeline" and a description: "Stay updated with ongoing and upcoming events, banners, and rewards."
        *   It features a calendar icon.
        *   It has the same hover effect as the Operator Database card.

    *   **Join the Community (Large Card):**
        *   This card spans two columns.
        *   It has the title "Join the Community" and a description: "Connect with fellow Doctors. Share strategies, discuss meta, and get help."
        *   It contains three buttons with logos for "Discord", "Reddit", and "Twitter", which likely link to the respective communities.

---

## Settings Page

### Purpose

The Settings Page allows logged-in users to manage their profile and preferences.

### Layout

The page has a single-column layout with a maximum width, centering the content on the page. The settings are grouped into cards.

### Colors and Theme

The page follows the general theme of the website. It makes use of `Card` components which have a slightly different background color from the main page background.

### Components

The Settings Page consists of the following components:

*   **Header:**
    *   A large title "Settings" with a settings icon next to it.
    *   A short description: "Manage your profile and preferences".

*   **Privacy Settings Card:**
    *   A card with the title "Privacy Settings" and description "Control how your profile appears to other users".
    *   It contains a "Leaderboard Visibility" disclosure component.
        *   **Trigger:** A clickable area that shows the current visibility status ("Your profile is visible..." or "Your profile is hidden..."). It has a chevron icon that rotates 90 degrees when the disclosure is opened.
        *   **Content:** When opened, it reveals a "Public Profile" switch to toggle the leaderboard visibility on and off. There is also a detailed explanation of what this setting affects.

*   **Profile Management Card:**
    *   A card with the title "Profile Management" and description "Update your profile data from the game servers".
    *   It contains a "Refresh Profile Data" disclosure component.
        *   **Trigger:** Similar to the privacy disclosure, it shows the purpose of this section and has a rotating chevron icon.
        *   **Content:** When opened, it provides a detailed explanation of what data will be refreshed and a "Refresh Profile Now" button. When the button is clicked, it shows a loading spinner and a "Refreshing Profile..." text until the process is complete.

*   **Coming Soon Card:**
    *   A dashed border card with the title "Coming Soon" and a description indicating that more settings and features will be added in the future. This card is visually distinct to indicate its placeholder nature.

If the user is not logged in, the page displays a simple card asking them to log in to access their settings. While the page is loading the user's authentication status, a spinning loader is displayed.

---

## Admin Page

### Purpose

The Admin Page is a restricted area for authorized administrators to manage the website's content and view statistics. Access is limited to users with specific admin roles (e.g., "super_admin", "tier_list_admin"). Unauthorized users are redirected to a 404 page.

### Layout

The page has a wide, single-column layout to accommodate large tables and data grids. The content is enclosed in a single large card with a border.

### Colors and Theme

The page uses the general website theme, but with more emphasis on data-dense components like tables and stat cards. It uses colored badges to represent different user roles, servers, and tier list statuses, making it easy to distinguish them at a glance.

### Components

The Admin Page is a comprehensive dashboard composed of several sections:

*   **Header:**
    *   A welcome message with the admin's nickname (e.g., "Welcome back, Admin!").
    *   The admin's role is displayed as a colored badge (e.g., `super_admin`).
    *   A "Refresh" button to reload all the data on the page.

*   **Stats Grid:**
    *   A grid of four cards displaying key statistics:
        1.  **Total Users:** Shows total user count, percentage growth, and a breakdown of users by role.
        2.  **Tier Lists:** Shows the total number of tier lists, percentage change, and the number of active lists.
        3.  **Total Versions:** Shows the total number of published tier list versions and percentage change.
        4.  **Operator Placements:** Shows the total number of operators placed across all tier lists and percentage change.
    *   Each card includes an icon, the main number, and a colored indicator for positive or negative trends.

*   **User Management Table:**
    *   A full-featured table for managing users.
    *   **Columns:** User (Avatar and Nickname), UID, Level, Server, Role, and Created date.
    *   **Functionality:**
        *   **Sorting:** All columns are sortable.
        *   **Filtering:** Users can be filtered by role and server, and searched by nickname or UID.
        *   **Pagination:** The table is paginated with controls to navigate through the user list and change the number of items shown per page.
        *   **Badges:** Roles and servers are displayed as colored badges for easy identification.

*   **Tier List Management:**
    *   This is the most complex part of the Admin Page, allowing full control over tier lists. It has two main views: a list view and an editor view.
    *   **List View (`TierListsTable`):**
        *   A table showing all tier lists with columns for Name, Status (Active/Inactive), number of Tiers, Operators, and Versions, and the last Updated date.
        *   It has sorting, filtering, and pagination, similar to the User Management Table.
        *   Each tier list has a dropdown menu with actions to "View", "Edit", or "Delete" it.
        *   A "Create" button opens a dialog to create a new tier list.
    *   **Editor View (`TierListEditor`):**
        *   A full-screen, drag-and-drop editor for a single tier list.
        *   **Settings:** Admins can edit the tier list's name, description, and active status.
        *   **Tiers:** Tiers can be added, deleted, and reordered by dragging. Each tier's name, color, and description can be edited.
        *   **Operators:** Operators can be dragged between tiers or within a tier to reorder them. New operators can be added from a searchable list of all available operators in the game.
        *   **Saving and Publishing:** Changes must be saved explicitly. A "Publish Version" button allows creating a versioned snapshot of the tier list with a changelog.

*   **Recent Activity:**
    *   A card that displays a feed of the most recent changes made to any tier list, such as an operator being moved, a tier being renamed, etc. It shows the type of change, the operator and tier list involved, who made the change, and when.

If the admin data is loading, a spinning loader is displayed in the center of the page.

---

## Operator Detail Page

### Purpose

This page provides a comprehensive and detailed view of a single Arknights operator, including their stats, skills, skins, voice lines, and lore. It's designed to be a one-stop-shop for all information related to a specific operator.

### Layout

The page has a visually striking layout dominated by a large hero section at the top. The main content below the hero section is organized into a tabbed interface, which is a vertical sidebar on desktop and a horizontal scrolling list on mobile.

### Colors and Theme

The page makes heavy use of the operator's artwork. The hero section has a parallax effect with the operator's main art. The rest of the page follows the website's general theme, with the accent color used to highlight active tabs and other interactive elements. Rarity is also visually represented with specific colors for glow effects and stars.

### Components

#### Hero Section

*   **Parallax Background:** A large, high-quality image of the operator's artwork is displayed at the top of the page. As the user scrolls, this image recedes and scales, creating a sense of depth.
*   **Operator Name and Rarity:** The operator's name is displayed in a very large font, with their rarity represented by a series of stars (e.g., ★★★★★★) in a color corresponding to their rarity.
*   **Tags and Badges:** Badges for the operator's profession (e.g., Caster), position (Melee/Ranged), and faction are displayed below the name.
*   **Breadcrumb:** A navigation link shows the path from "Operators" to the current operator.
*   **Faction Logo:** The logo of the operator's faction is displayed on the right side of the hero section on larger screens.

#### Tabs Section

Below the hero section, the content is divided into six tabs:

1.  **Info Tab:**
    *   This is the default and most complex tab, acting as a "stat calculator".
    *   **Operator Controls:** A set of interactive controls allows the user to select the operator's Promotion level, Potential, Level, Trust, and Module.
    *   **Combat Stats:** A grid of stat cards dynamically updates to show the operator's stats (Health, ATK, DEF, etc.) based on the selected controls.
    *   **Attack Range:** A visual grid displays the operator's attack range, which also updates based on the selected promotion.
    *   **Talents & Profile:** Collapsible sections show the operator's talents (with descriptions that update based on stats), and profile information (like Place of Birth, Race, etc.).
    *   **Module Details:** If a module is selected, another collapsible section appears with details about the module's stat bonuses and effects.

2.  **Skills Tab:**
    *   This tab allows for in-depth analysis of the operator's skills.
    *   **Skill Selection:** Users can switch between the operator's skills.
    *   **Single Level View:** A slider lets the user see a skill's description and stats (SP Cost, Duration, etc.) at any specific level.
    *   **Comparison View:** For skills with Mastery levels, users can select multiple levels to compare them side-by-side, with an option to highlight only the differences between levels.
    *   **Talents:** A collapsible section at the bottom reiterates the operator's talents.

3.  **Level-Up Costs Tab:**
    *   This tab is a reference for the materials needed to upgrade the operator. It has its own set of sub-tabs:
        *   **Elite Promotion:** Shows the materials needed for each promotion (Elite 1, Elite 2).
        *   **Skill Mastery:** Shows the materials needed for each skill mastery level (M1, M2, M3).
        *   **Modules:** Shows the materials needed to unlock and upgrade each of the operator's modules.

4.  **Skins Tab:**
    *   A gallery for viewing the operator's different outfits.
    *   **Skin Viewer:** A large area displays the artwork of the selected skin, with a fullscreen option.
    *   **Skin Selector:** A list of thumbnails allows the user to switch between different skins.
    *   **Skin Details:** A panel shows information like the skin's artist and how to obtain it.
    *   **Chibi Viewer:** An interactive viewer displays the operator's animated in-game "chibi" sprite, which updates to match the selected skin. It allows playing different animations (like "Idle" or "Attack") and changing the viewing angle.

5.  **Audio Tab:**
    *   A player for listening to the operator's voice lines.
    *   **Language Selection:** A dropdown to choose the voice-over language (e.g., Japanese, English).
    *   **Voice Line List:** Voice lines are grouped into categories (e.g., Greetings, Battle) and displayed in a list. Each entry has a play/pause button, the transcribed text, and a progress bar when playing.

6.  **Lore Tab:**
    *   Displays the operator's background story and archive files.
    The lore is presented in collapsible sections (e.g., "Clinical Analysis," "Promotion Record").
    *   The text is intelligently parsed and styled, with headers, quotes, lists, and technical terms being visually distinct, making it look like a real document.

---

## Operator List Page

### Purpose

This page allows users to browse, search, sort, and filter the entire database of Arknights operators. It's designed for efficient discovery and comparison of operators.

### Layout

The page features a main content area with a header, a set of controls for searching and filtering, and the main grid or list of operators. The layout is highly interactive and responsive.

### Colors and Theme

The page follows the general website theme. The accent color is used for active toggles and highlights. Rarity is consistently represented by specific colors on the operator cards. A unique hover effect on the operator cards uses a grayscale effect to draw focus to the hovered operator.

### Components

*   **Header:**
    *   A large title "Operators" and a brief description.

*   **Search and Controls:**
    *   **Search Bar:** A prominent search bar at the top allows users to quickly find operators by name.
    *   **View Mode Toggle:** A toggle switch allows users to switch between a **Grid View** and a **List View**. The toggle itself has a smooth sliding animation.
    *   **Sort Controls:** Dropdowns and buttons to sort the operators by various criteria (e.g., Name, Rarity) in ascending or descending order.
    *   **Filter Button:** A button that opens the filter menu. It displays a badge with the number of active filters. On desktop, this opens a popover; on mobile, it opens a full-screen dialog.

*   **Filter Menu (`OperatorFilters`):**
    *   A comprehensive set of filters to narrow down the operator list.
    *   **Basic Filters:** For `Class` (with icons) and `Rarity` (e.g., 6★, 5★).
    *   **Advanced Filters:** A collapsible section with dropdown menus for more specific filtering, including `Archetype`, `Nation`, `Faction`, `Race`, `Gender`, and `Artist`.
    *   A "Clear all" button allows users to reset all filters.

*   **Operator Display:**
    *   The operators are displayed in either a grid or a list, depending on the selected view mode.
    *   **Grid View (`OperatorCardGrid`):**
        *   Displays operators as compact cards with their portrait, name, and a colored rarity indicator.
        *   Hovering over a card reveals more details in a popover, including archetype, position, and race.
    *   **List View (`OperatorCardList`):**
        *   Displays operators in a list. On desktop, this can be a single wide column with detailed info (Name, Rarity, Class, etc.) or a multi-column compact list. On mobile, it's always a compact list.
    *   **Hover Effect:** In both views, hovering over an operator card makes all other cards turn to grayscale, which puts the focus on the hovered operator.

*   **Pagination:**
    *   The list of operators is paginated, with navigation controls at the bottom of the page to move between pages.

*   **Empty State:**
    *   If no operators match the current search and filter criteria, a message is displayed, along with a button to clear all filters.

---

## Tier List Page

### Purpose

This page serves two functions. First, it acts as an index for all available operator tier lists. Second, it provides a detailed view of a specific tier list, showing the ranking of operators across different tiers.

### Layout

The page has two distinct layouts depending on whether the user is viewing the index or a specific tier list. Both layouts are clean and organized.

### Colors and Theme

The page follows the general website theme. The tier list detail view uses a distinct color for each tier label (e.g., S, A, B), with the text color on the label automatically adjusting for contrast, making the tiers easily distinguishable.

### Components

#### Index Mode

When no specific tier list is selected, the page displays an index of all available tier lists.

*   **Header:** A title "Tier Lists" and a count of how many lists are available.
*   **Tier List Grid:** A grid of cards, each representing a single tier list.
    *   **Tier List Card:**
        *   A clickable card that links to the detailed view of that tier list.
        *   The top of the card has a header that shows a preview of the top-ranked operators in that list.
        *   The main body of the card displays the tier list's name, description, the number of operators and tiers it contains, and the date it was last updated.

#### Detail Mode

When a specific tier list is selected, the page displays its detailed contents.

*   **Header:**
    *   The name and description of the tier list.
    *   A "Changelog" button that shows the version history of the list. Clicking it opens a dialog with a list of all published versions. Clicking a version opens another dialog with the detailed changelog for that version.
    *   An "Info" button that opens a popover explaining the ranking criteria and how the tier list works.
*   **Tier Rows:**
    *   The main content is a list of rows, one for each tier (e.g., S+, S, A).
    *   **Tier Label:** On the left of each row is a large, colored label with the tier's name (e.g., "S"). If the tier has a description, clicking this label opens a dialog with more information.
    *   **Operators Grid:** To the right of the label is a grid of operator cards, showing all operators ranked in that tier.
    *   **Operator Card (`TierOperatorCard`):**
        *   A compact card showing the operator's portrait and a colored bar for their rarity.
        *   Hovering over the card reveals the operator's name and a detailed tooltip with their archetype, rarity, faction, and other info.
        *   The same grayscale hover effect from the Operator List page is used here to highlight the hovered operator.
*   **Empty State:**
    *   If a tier has no operators, a message is displayed in that row.

---

## Recruitment Calculator Page

### Purpose

This page is a utility for the game Arknights that helps players determine the possible operators they can get from the in-game recruitment system based on a combination of tags.

### Layout

The page has a clean, tool-oriented layout. It's divided into three main sections: a header, a tag selection area, and a results area.

### Colors and Theme

The page follows the general website theme. It uses special colors to highlight important recruitment tags: orange for "Top Operator" (which guarantees a 6-star operator) and yellow for "Senior Operator" (which guarantees a 5-star operator). The results are also color-coded based on the rarity of the operators they can produce.

### Components

*   **Header:**
    *   A title "Recruitment Calculator" with a calculator icon.
    *   A description explaining that users can select up to 5 tags to see the possible outcomes.

*   **Tag Selector:**
    *   A large area where all the possible recruitment tags are displayed as buttons.
    *   The tags are grouped by type (e.g., "Class", "Position", "Affix") in collapsible sections.
    *   Users can click on the tags to select them. A counter shows how many tags are currently selected (up to a maximum of 5).
    *   Selected tags are visually highlighted. Once 5 tags are selected, all other tags are disabled.
    *   A "Reset" button allows the user to clear all selected tags.

*   **Results Section:**
    *   This section dynamically updates as the user selects and deselects tags.
    *   **Filter Options:** A set of controls allows the user to refine the results, such as a switch to include or exclude 1-star "Robot" operators.
    *   **Results List:**
        *   The main part of this section is a list of all possible outcomes from the selected tags.
        *   The results are grouped into "High-Value Combinations" (those that guarantee a 5 or 6-star operator) and "Other Combinations".
        *   **Combination Result:** Each possible outcome is displayed in its own collapsible row.
            *   The row is highlighted in orange for a guaranteed 6-star or yellow for a guaranteed 5-star.
            *   The header of the row shows the tags in that combination and the rarity range of the possible operators (e.g., "5★~6★").
            *   Expanding the row reveals a grid of all the operators that can be recruited with that tag combination.
            *   **Operator Card:** Each operator is shown as a small card with their portrait, name, rarity, and class icon.
    If no tags are selected, this area prompts the user to select tags to see the results.

---

## User Profile Page

### Purpose

This page displays the in-game profile of a specific user, including their owned operators, inventory, and base setup. It's a public-facing page that other users can view.

### Layout

The page has a header section followed by a tabbed interface for the main content. The layout is clean and focused on presenting the user's data in an organized way.

### Colors and Theme

The page follows the general website theme. It uses progress bars and colored badges to provide at-a-glance information about the user's operators and base.

### Components

*   **User Header:**
    *   A large card at the top of the page that displays the user's main profile information.
    *   **Avatar:** The user's in-game assistant (secretary) is shown as their avatar.
    *   **User Info:** Displays the user's nickname, ID number, level, and in-game status message. A button allows copying the full username to the clipboard.
    *   **Stats:** Shows the user's current LMD, Orundum, Originium, and friend limit, with the numbers animating as they count up.

*   **Tabs Section:**
    Below the header, the content is divided into three tabs:

    1.  **Characters Tab:**
        *   Displays the user's entire roster of owned operators.
        *   **Filters:** Controls at the top allow the user to sort the operators (by Level, Rarity, etc.), filter by rarity, and search by name.
        *   **View Mode:** A toggle allows switching between a "Detailed View" and a "Compact View".
        *   **Detailed View:** Shows a grid of large cards, each with the operator's artwork, name, rarity, level, stats, and collapsible sections for skills and modules.
        *   **Compact View:** Shows a grid of smaller, more compact cards that display a lot of information (level, potential, skill mastery, modules) as small icons and badges around the operator's avatar, similar to the in-game UI.
        *   **Infinite Scroll:** More operators are automatically loaded as the user scrolls down the page.
        *   **Dialog:** Clicking on any operator card (in either view) opens a full-screen dialog with even more detailed information about that specific operator, including a parallax header and detailed breakdowns of their stats, skills, and modules.

    2.  **Items Tab:**
        *   Displays the user's inventory of items.
        *   The items are shown in a table-like list that can be sorted by name or quantity.
        *   A search bar allows filtering the items.
        *   Clicking on an item opens a dialog with detailed information about it, including its description, usage, and where to farm it.

    3.  **Base Tab:**
        *   Displays information about the user's in-game base.
        *   This section is marked as a "work-in-progress".
        *   It shows a grid of cards for the user's **Trading Posts** and **Factories**, displaying their level, current product or strategy, and efficiency as a percentage and a progress bar.
