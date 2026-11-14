import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "apps.authorized.title": {
        text: "Authorized applications",
        description: "Card title in Settings over the list of other sites the visitor has signed in to with their myrtle.moe account (OAuth).",
    },
    "apps.authorized.desc": {
        text: "Sites you signed in to with your myrtle.moe account. Each can see who you are, and nothing else.",
        description: "Card description under that title.",
    },
    "apps.authorized.empty": {
        text: "Nothing here yet. A site you sign in to with myrtle.moe shows up in this list.",
        description: "Empty state of the authorized applications list.",
    },
    "apps.authorized.loadFailed": {
        text: "Couldn't load your authorized applications.",
        description: "Error line when the authorized applications list failed to load. A retry button follows.",
    },
    "apps.authorized.by": {
        text: "Registered by {owner}",
        description: "Line under an application's name. {owner} is the in-game name of the player who registered it.",
    },
    "apps.authorized.since": {
        text: "Authorized {date}",
        description: "When the visitor authorized an application. {date} is a calendar date.",
    },
    "apps.authorized.lastUsed": {
        text: "last used {when}",
        description: "When an application last used its access, after the authorized date and a separator. {when} is a relative time such as '2 d ago'.",
    },
    "apps.authorized.revoke": {
        text: "Remove access",
        description: "Button on an authorized application: withdraw the visitor's authorization so the site can no longer see who they are.",
    },
    "apps.authorized.revokeLabel": {
        text: "Remove access for {app}",
        description: "Accessible name of that button. {app} is the application's name.",
    },
    "apps.toast.revoked.title": {
        text: "Access removed",
        description: "Success toast title after withdrawing an application's authorization.",
    },
    "apps.toast.revoked.body": {
        text: "{app} can no longer see who you are.",
        description: "Success toast body after withdrawing an application's authorization. {app} is its name.",
    },
    "apps.toast.revokeFailed.title": {
        text: "Couldn't remove access",
        description: "Error toast title when withdrawing an authorization failed. The body explains why.",
    },
    "apps.retry": {
        text: "Try again",
        description: "Button that reloads a list that failed to load.",
    },

    "apps.dev.title": {
        text: "Your applications",
        description: "Card title in Settings over the OAuth applications the visitor registered as a developer.",
    },
    "apps.dev.desc": {
        text: "Let people sign in to your own site with their myrtle.moe account. {docs} has the endpoints and a working example.",
        description: "Card description under that title. {docs} is a link whose text is apps.dev.docsLink; move it wherever the sentence needs it.",
    },
    "apps.dev.docsLink": {
        text: "The developer guide",
        description: "Link text substituted into apps.dev.desc as {docs}. It opens the OAuth guide on GitHub.",
    },
    "apps.dev.empty": {
        text: "You haven't registered an application.",
        description: "Empty state of the visitor's own applications list.",
    },
    "apps.dev.loadFailed": {
        text: "Couldn't load your applications.",
        description: "Error line when the visitor's own applications failed to load. A retry button follows.",
    },
    "apps.dev.new": {
        text: "New application",
        description: "Button that opens the form to register an OAuth application.",
    },
    "apps.dev.limit": {
        text: "You've reached the limit of {max} applications.",
        description: "Note shown in place of the New application button once the account has the most it may register.",
    },
    "apps.dev.clientId": {
        text: "Client ID",
        description: "Label of an application's public identifier, an OAuth term developers know as 'client ID'.",
    },
    "apps.dev.redirectUris": {
        text: "Redirect URIs",
        description: "Label of the list of addresses an application may send players back to, an OAuth term.",
    },
    "apps.dev.users": {
        text: "{count, plural, =0 {No one has authorized it yet} one {# person has authorized it} other {# people have authorized it}}",
        description: "How many players currently have this application authorized.",
    },
    "apps.dev.edit": {
        text: "Edit",
        description: "Button on one of the visitor's applications: change its name, description and redirect URIs.",
    },
    "apps.dev.rotate": {
        text: "New secret",
        description: "Button on one of the visitor's applications: replace its client secret with a new one.",
    },
    "apps.dev.delete": {
        text: "Delete",
        description: "Button on one of the visitor's applications: delete it.",
    },
    "apps.copy": {
        text: "Copy",
        description: "Button that copies the value next to it (a client ID or a client secret) to the clipboard.",
    },
    "apps.copied": {
        text: "Copied",
        description: "Label the Copy button shows for a moment after copying.",
    },
    "apps.copyLabel": {
        text: "Copy {what}",
        description: "Accessible name of a Copy button. {what} is the label of the value it copies, such as 'Client ID'.",
    },

    "apps.form.createTitle": {
        text: "New application",
        description: "Title of the dialog that registers an OAuth application.",
    },
    "apps.form.editTitle": {
        text: "Edit application",
        description: "Title of the dialog that edits an OAuth application.",
    },
    "apps.form.intro": {
        text: "People see the name and description, and your in-game name, when they are asked to authorize it.",
        description: "Intro line of the application form dialog.",
    },
    "apps.form.name": {
        text: "Name",
        description: "Label of the application name field.",
    },
    "apps.form.namePlaceholder": {
        text: "My site",
        description: "Placeholder of the application name field.",
    },
    "apps.form.description": {
        text: "Description",
        description: "Label of the application description field. Optional.",
    },
    "apps.form.descriptionHint": {
        text: "Optional. One line about what the site is.",
        description: "Hint under the application description field.",
    },
    "apps.form.redirectUris": {
        text: "Redirect URIs",
        description: "Label of the redirect URIs field, an OAuth term: the addresses players may be sent back to.",
    },
    "apps.form.redirectHint": {
        text: "One per line, up to {max}. Each must be an https address; http is allowed for localhost. A sign-in request has to name one of these exactly.",
        description: "Hint under the redirect URIs field. {max} is a number.",
    },
    "apps.form.nameRequired": {
        text: "Give the application a name.",
        description: "Validation message when the application name is empty.",
    },
    "apps.form.redirectRequired": {
        text: "Add at least one redirect URI.",
        description: "Validation message when no redirect URI was entered.",
    },
    "apps.form.cancel": {
        text: "Cancel",
        description: "Button that closes the application form without saving.",
    },
    "apps.form.create": {
        text: "Create application",
        description: "Submit button of the form that registers an application.",
    },
    "apps.form.save": {
        text: "Save changes",
        description: "Submit button of the form that edits an application.",
    },
    "apps.toast.saved.title": {
        text: "Application saved",
        description: "Success toast title after editing an application.",
    },
    "apps.toast.saved.body": {
        text: "{app} is up to date.",
        description: "Success toast body after editing an application. {app} is its name.",
    },

    "apps.secret.createdTitle": {
        text: "Application created",
        description: "Title of the dialog shown right after registering an application, which reveals its credentials.",
    },
    "apps.secret.rotatedTitle": {
        text: "New client secret",
        description: "Title of the dialog shown right after replacing an application's client secret.",
    },
    "apps.secret.body": {
        text: "Copy the client secret now and keep it on your server. It is stored only as a hash, so this is the one time it can be shown.",
        description: "Body of the dialog that reveals a client secret. 'Hash' is the one-way fingerprint of the secret.",
    },
    "apps.secret.rotatedNote": {
        text: "The previous secret has stopped working. People who already signed in stay signed in.",
        description: "Extra line in that dialog after a secret was replaced.",
    },
    "apps.secret.label": {
        text: "Client secret",
        description: "Label of an application's secret credential, an OAuth term developers know as 'client secret'.",
    },
    "apps.secret.done": {
        text: "I've saved it",
        description: "Button that closes the dialog revealing a client secret.",
    },

    "apps.rotate.title": {
        text: "Replace the client secret?",
        description: "Title of the confirmation before replacing an application's client secret.",
    },
    "apps.rotate.body": {
        text: "{app} stops being able to sign people in until you put the new secret on your server. People who already signed in stay signed in.",
        description: "Body of that confirmation. {app} is the application's name.",
    },
    "apps.rotate.confirm": {
        text: "Replace secret",
        description: "Confirm button of that confirmation.",
    },
    "apps.toast.rotateFailed.title": {
        text: "Couldn't replace the secret",
        description: "Error toast title when replacing a client secret failed. The body explains why.",
    },

    "apps.delete.title": {
        text: "Delete {app}?",
        description: "Title of the confirmation before deleting an application. {app} is its name.",
    },
    "apps.delete.body": {
        text: "Everyone who signed in with it is signed out, and its client ID and secret stop working. This can't be undone.",
        description: "Body of that confirmation.",
    },
    "apps.delete.confirm": {
        text: "Delete application",
        description: "Confirm button of that confirmation.",
    },
    "apps.toast.deleted.title": {
        text: "Application deleted",
        description: "Success toast title after deleting an application.",
    },
    "apps.toast.deleted.body": {
        text: "{app} and its access are gone.",
        description: "Success toast body after deleting an application. {app} is its name.",
    },
    "apps.toast.deleteFailed.title": {
        text: "Couldn't delete the application",
        description: "Error toast title when deleting an application failed. The body explains why.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
