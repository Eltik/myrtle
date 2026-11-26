import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "authDialog.title": {
        text: "Sign in",
        description: "Title of the sign-in dialog.",
    },
    "authDialog.description.yostar": {
        text: "Sign in with a one-time code sent to your Yostar email. The code is used once and never stored.",
        description: "Dialog subtitle for the global servers. 'Yostar' is the publisher's name. It must stay true: the code is never stored, and what the Keep me synced box below stores is said under that box.",
    },
    "authDialog.description.bili": {
        text: "Sign in with your Bilibili account. Your password is used once to sign in and never stored.",
        description: "Dialog subtitle for the Bilibili server. 'Bilibili' is the Chinese publisher's name. It must stay true: the password is never stored.",
    },
    "authDialog.description.cn": {
        text: "Experimental: signs in with your Hypergryph (CN) account through the same passport flow the Skland app uses. Untested against a real account, so it may fail even with correct credentials. Your password or code is used once to sign in and never stored.",
        description: "Dialog subtitle for the Chinese server. 'Hypergryph' is the developer and 'Skland' its companion app; both stay as-is.",
    },
    "authDialog.email": {
        text: "Email",
        description: "Field label for the visitor's YoStar email address.",
    },
    "authDialog.code": {
        text: "Code",
        description: "Field label for the verification code the visitor received.",
    },
    "authDialog.oneTimePassword": {
        text: "One-time password",
        description: "Accessible name of the six-box verification-code field as a whole.",
    },
    "authDialog.otpCharacter": {
        text: "Character {index} of {total}",
        description: "Accessible name of one box in the verification-code field. Both values are plain numbers.",
    },
    "authDialog.password": {
        text: "Password",
        description: "Tab and field label for password sign-in.",
    },
    "authDialog.smsCode": {
        text: "SMS code",
        description: "Tab label for signing in with a code texted to the visitor's phone.",
    },
    "authDialog.username": {
        text: "Username",
        description: "Field label for the Bilibili account name.",
    },
    "authDialog.usernamePlaceholder": {
        text: "Bilibili username, phone, or email",
        description: "Placeholder in the Bilibili username field, listing what the field accepts.",
    },
    "authDialog.phone": {
        text: "Phone",
        description: "Field label for a phone number.",
    },
    "authDialog.biliPhonePlaceholder": {
        text: "Bilibili account phone number",
        description: "Placeholder in the phone field on the Bilibili SMS tab.",
    },
    "authDialog.cnPhonePlaceholder": {
        text: "Hypergryph account phone number",
        description: "Placeholder in the phone field for the Chinese server.",
    },
    "authDialog.biliSmsExperimental": {
        text: "Experimental: Bilibili SMS sign-in hasn't been tested with a real account, so it may fail even with a correct code.",
        description: "Warning above the Bilibili SMS form that the flow is untested.",
    },
    "authDialog.server": {
        text: "Server",
        description: "Visible field label over the game-server picker in the sign-in dialog. A 'server' is a regional release of the game (Global, Japan, Korea, China).",
    },
    "authDialog.selectServer": {
        text: "Select server",
        description: "Accessible name of the game-server picker. The option names come from game data and are not translated here.",
    },
    "authDialog.cancel": {
        text: "Cancel",
        description: "Button that closes the sign-in dialog without signing in.",
    },
    "authDialog.sending": {
        text: "Sending…",
        description: "Submit-button label while the verification code is being sent. Ends in an ellipsis character.",
    },
    "authDialog.sendCode": {
        text: "Send code",
        description: "Submit-button label that requests a verification code.",
    },
    "authDialog.loggingIn": {
        text: "Signing in…",
        description: "Submit-button label while the sign-in request is in flight. Ends in an ellipsis character.",
    },
    "authDialog.login": {
        text: "Sign in",
        description: "Submit-button label that completes sign-in.",
    },
    "authDialog.resendIn": {
        text: "Resend available in {seconds}s",
        description: "Cooldown note under the submit button. {seconds} is a plain number and 's' abbreviates seconds.",
    },
    "authDialog.resendCode": {
        text: "Resend code",
        description: "Link that requests a fresh verification code once the cooldown has passed.",
    },
    "authDialog.changeEmail": {
        text: "Change email",
        description: "Link that clears the entered code so a different email address can be used.",
    },
    "authDialog.changePhone": {
        text: "Change phone",
        description: "Link that clears the entered code so a different phone number can be used.",
    },
    "authDialog.toast.sentCode": {
        text: "Code sent",
        description: "Success toast title after a verification code goes out.",
    },
    "authDialog.toast.sentOtpEmail": {
        text: "Check your email for the code.",
        description: "Success toast body after emailing a one-time password.",
    },
    "authDialog.toast.sentSms": {
        text: "Check your phone for the code.",
        description: "Success toast body after texting a verification code.",
    },
    "authDialog.toast.error": {
        text: "Couldn't send the code",
        description: "Error toast title used when sending a sign-in code fails.",
    },
    "authDialog.toast.otpError": {
        text: "{error}",
        description: "Error toast body when the one-time code could not be emailed. {error} is the server's own message; keep it as the whole body.",
    },
    "authDialog.toast.codeError": {
        text: "{error}",
        description: "Error toast body when an SMS code could not be sent. {error} is the server's own message; keep it as the whole body.",
    },
    "authDialog.toast.loggedIn": {
        text: "Signed in",
        description: "Success toast title after signing in.",
    },
    "authDialog.saveCredentials": {
        text: "Keep me synced",
        description: "Label of the checkbox in the login dialog that decides whether the site may store the player's GAME credentials. It does not affect the myrtle.moe session itself, which lasts a week either way, so the wording must not promise to keep the player signed in.",
    },
    "authDialog.saveCredentials.hint": {
        text: "Keeps an encrypted game token on our server, so re-syncing works without signing in again. Uncheck it and nothing is saved, so a re-sync more than an hour later asks you to sign in again. Revoke it any time in Settings.",
        description: "Caption under that checkbox, saying exactly what it stores: an encrypted game access token, never the password or the code. 'Settings' is this site's settings page, where the stored token can be disconnected.",
    },
    "authDialog.toast.loginFailed": {
        text: "Couldn't sign in",
        description: "Error toast title when signing in was rejected.",
    },
    "authDialog.toast.loginRejected": {
        text: "Check the code and try again, or request a new one.\n{detail}",
        description: "Error toast body when the login server turned the credentials down. {detail} is the server's own reason, on its own line.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
