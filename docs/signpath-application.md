# SignPath application preparation

Prepared 2026-09-11; application fields inspected 2026-09-13. SignPath is the preferred route to investigate. No application has been submitted, terms accepted, account connected or certificate approved. This is an internal preparation note, not a public code signing policy or a claim of sponsorship.

## Project details

- Project: Villow Setup.
- Maintainer: GitHub user `nicholasnucifora`, a solo developer living in Australia. Enter your own contact details directly in SignPath's form.
- Intended distribution repository: https://github.com/nicholasnucifora/Villow-Setup.
- Current development location: the root of the standalone `nicholasnucifora/Villow-Setup` checkout. Confirm the reviewed source has been committed and pushed before using the public repository as enrollment evidence.
- Parent application source repository: https://github.com/nicholasnucifora/Villow.
- Purpose: a local Windows application that guides a person through deploying their own Villow instance using their Vercel, Supabase and Google Cloud accounts. Villow is an intentional video-watching interface using YouTube APIs and embeds, without affiliation with Google or YouTube.
- Implementation: Tauri 2, Rust and React/TypeScript; Windows x64 application and NSIS installer, version 0.1.0.
- License context: Setup now carries the parent's MIT notice in its source and installer resources. The [dependency inventory](dependency-license-review.md) records 370 selected Windows Cargo packages and six npm packages; 12 packages need notice-location review, and final binary/component obligations remain unqualified.
- Current release state: unsigned local development installer; packaging/startup and local integration tested. Public signed Windows and real-provider deployment qualification remain outstanding. No public Setup release or established Setup reputation has been verified.

## Suggested eligibility enquiry

Use this text for an eligibility enquiry after reviewing it yourself. SignPath's [official contact page](https://signpath.io/contact) lists `info@signpath.io` and a contact form; specify that the question concerns the Foundation open-source program. The [application page](https://signpath.org/apply.html) is an enrollment form with required agreement checkboxes, not a neutral enquiry form. Do not submit that form merely to ask about eligibility before you are ready to accept its requirements.

> I maintain Villow and am preparing Villow Setup, a Windows installer application that helps users deploy their own instance into their own cloud accounts. I am a solo developer based in Australia. Setup is developed in https://github.com/nicholasnucifora/Villow-Setup, with web application source at https://github.com/nicholasnucifora/Villow. Setup has an unsigned development installer but has not yet had a qualified public release. The source is MIT licensed; signed packaging and component notices are still being prepared.
>
> Before I apply for Foundation signing, could you confirm whether a project at this stage is eligible, or what public release/reputation evidence you would need first? Can one solo maintainer hold the author, reviewer and signing-approver roles? Your conditions page is marked Draft; which version would govern enrollment?

## What the actual application asks for

The embedded form on the [official application page](https://signpath.org/apply.html) was readable on 2026-09-13. No fields were filled and no consent was given. Prepare these answers once the project is eligible and the public links actually contain the reviewed source and documents:

| Field | Prepared answer or remaining work |
| --- | --- |
| Project Name (required) | Villow Setup. The form expects the name to identify the project in search results. |
| Repository URL (required) | `https://github.com/nicholasnucifora/Villow-Setup`, after confirming the complete reviewed source is committed and publicly visible there. |
| Homepage URL (required) | The repository page is allowed; a separate website is not required by this form. |
| Download URL | A real release/download page, when available. The form requests a Foundation attribution there; ask how to handle this before acceptance instead of claiming sponsorship prematurely. |
| Privacy Policy URL | Publish the reviewed [privacy notice](privacy.md) and provide its public link. The form requires a policy when software collects user data. A local file path is not suitable. |
| Wikipedia URL | Optional; leave blank if there is no article. |
| Tagline (required) | A Windows app that guides people through setting up their own Villow instance in cloud accounts they control. |
| Description (required) | Villow Setup helps individuals deploy and manage initial setup of their own Villow instance. Villow provides an intentional video-watching interface using YouTube APIs and embedded playback. Setup guides account selection, deployment and recovery while the user retains control of their hosting accounts. It is independent software with no Google or YouTube affiliation. |
| Reputation (required) | Supply genuine public evidence such as usage/download statistics, project activity or independent discussions. No qualifying evidence has been verified for Setup; do not invent claims or present local tests as adoption. Ask whether the current stage is eligible first. |
| Maintainer Type / Build System | Disclose solo individual maintenance and that GitHub Actions is the intended build system. Hosted signing has not run yet. The dropdown choices could not be opened by the inspection tool, so exact option labels are not prescribed here. |
| First Name / Last Name / Email (required) | Enter your own account/contact details directly. |
| Company Name | Optional and only applicable if you have one. Do not invent an organization. |
| Primary Discovery Channel (required) | Choose the truthful source. The optional exact-source field can say ChatGPT/Codex if this conversation was your first introduction. |

The form has required consent for its Code of Conduct (including Foundation-named certificates and possible revocation) and required consent to store/process your personal data. Additional marketing communications are a separate optional checkbox. Read the linked terms and privacy policy yourself before accepting; the page's Draft wording does not remove these explicit agreement requirements. This is a description of the form, not a legal interpretation of those terms.

## Work owned by the maintainer and implementation work

The maintainer handles contact information, account security, enrollment decisions and eventual release approvals. Enable GitHub MFA, retain recovery access privately, and enable SignPath MFA when an account is established. Never put account passwords, recovery codes or signing tokens in this document or chat.

Local preparation now includes the inherited license, [privacy notice](privacy.md), [draft Code signing policy](code-signing-policy.md), README links and a repeatable dependency inventory. Remaining implementation work includes complete third-party notices and integration of the approved signing service into the build. The existing certificate-store workflow does not implement SignPath yet. Do not add a Foundation sponsorship statement before approval.

For the technical onboarding discussion, confirm the supported process for this Tauri/NSIS package. Both the embedded application and outer installer must be signed, with the signed application packaged unchanged. Do not assume that signing the outer EXE also signs its payload. Confirm how the uninstaller and Microsoft runtime components fit the approved artifact configuration.

After acceptance, record the approved project configuration and use protected GitHub settings for any integration secrets. SignPath's Windows signatures and Villow's app-release authentication are separate mechanisms; both still need their own release checks.

The [current conditions](https://signpath.org/terms.html) remain the reference for eligibility and operating requirements. They require an existing release, verifiable reputation for executable projects and manual signing approval; acceptance is discretionary. A local draft or passing tests cannot establish approval.
