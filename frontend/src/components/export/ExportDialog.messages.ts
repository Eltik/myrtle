import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `components/export/**` shares the `skins` namespace, per the namespace table. */
export const namespace = "skins";

export const messages = {
    "export.title": {
        text: "Export {name}",
        description: "Title of the export dialog. {name} is the plural name of whatever is being exported, e.g. 'Operators'; it comes from the caller's export schema.",
    },
    "export.description": {
        text: "Choose a format, pick the fields you need, and download the file.",
        description: "Caption under the export dialog's title.",
    },
    "export.section.format": {
        text: "Format",
        description: "Section heading in the export dialog, above the file-format choices. Rendered uppercase.",
    },
    "export.section.scope": {
        text: "Scope",
        description: "Section heading in the export dialog, above the choice of how many rows to export. Rendered uppercase.",
    },
    "export.section.fields": {
        text: "Fields",
        description: "Section heading in the export dialog, above the per-column checkboxes. Rendered uppercase, with a selected/total count after it.",
    },
    "export.section.options": {
        text: "Options",
        description: "Section heading in the export dialog, above the format-specific switches. Rendered uppercase.",
    },
    "export.section.filename": {
        text: "Filename",
        description: "Section heading in the export dialog, above the file-name field. Rendered uppercase.",
    },
    "export.format.json.label": {
        text: "JSON",
        description: "File-format name. An acronym; normally left as-is.",
    },
    "export.format.json.desc": {
        text: "Structured, machine-readable; arrays/objects preserved",
        description: "One-line description of the JSON export format.",
    },
    "export.format.csv.label": {
        text: "CSV",
        description: "File-format name. An acronym; normally left as-is.",
    },
    "export.format.csv.desc": {
        text: "Comma-separated, opens in Excel / Sheets",
        description: "One-line description of the CSV export format. 'Excel' and 'Sheets' are product names and stay as-is.",
    },
    "export.format.tsv.label": {
        text: "TSV",
        description: "File-format name. An acronym; normally left as-is.",
    },
    "export.format.tsv.desc": {
        text: "Tab-separated, safest for fields containing commas",
        description: "One-line description of the TSV export format.",
    },
    "export.format.markdown.label": {
        text: "Markdown table",
        description: "File-format name: a table written in Markdown. 'Markdown' is a format name and stays as-is.",
    },
    "export.format.markdown.desc": {
        text: "Renders as a table in any Markdown viewer",
        description: "One-line description of the Markdown export format.",
    },
    "export.format.xml.label": {
        text: "XML",
        description: "File-format name. An acronym; normally left as-is.",
    },
    "export.format.xml.desc": {
        text: "Hierarchical, schema-friendly",
        description: "One-line description of the XML export format.",
    },
    "export.format.yaml.label": {
        text: "YAML",
        description: "File-format name. An acronym; normally left as-is.",
    },
    "export.format.yaml.desc": {
        text: "Human-readable, compact",
        description: "One-line description of the YAML export format.",
    },
    "export.scope.all": {
        text: "Entire collection",
        description: "Export scope: every row in the dataset, ignoring the current filters.",
    },
    "export.scope.filtered": {
        text: "Current filtered results",
        description: "Export scope: only the rows the visitor's filters currently match.",
    },
    "export.scope.page": {
        text: "Visible page only",
        description: "Export scope: only the rows on the page currently on screen.",
    },
    "export.fields.all": {
        text: "All",
        description: "Button that ticks every field checkbox in the export dialog.",
    },
    "export.fields.default": {
        text: "Default",
        description: "Button that restores the export dialog's default field selection.",
    },
    "export.fields.none": {
        text: "None",
        description: "Button that unticks every field checkbox in the export dialog.",
    },
    "export.fields.general": {
        text: "General",
        description: "Fallback heading for export fields that the schema files under no group of their own. The other group names come from the schema and are not translated here.",
    },
    "export.fields.toggleGroup": {
        text: "Toggle {group}",
        description: "Accessible name of the checkbox that ticks or unticks a whole field group. {group} is the group's name, which comes from the export schema.",
    },
    "export.option.pretty.label": {
        text: "Pretty print",
        description: "Switch label: indent the output rather than emitting it on one line.",
    },
    "export.option.pretty.desc": {
        text: "Indent JSON / XML for readability",
        description: "Caption under the 'Pretty print' switch.",
    },
    "export.option.indent.label": {
        text: "Indent",
        description: "Label of the dropdown choosing how far nested lines are indented. A noun.",
    },
    "export.option.indent.desc": {
        text: "Used by JSON and YAML",
        description: "Caption under the 'Indent' dropdown.",
    },
    "export.option.indent.none": {
        text: "None",
        description: "Indent dropdown option: no indentation at all.",
    },
    "export.option.indent.two": {
        text: "2 spaces",
        description: "Indent dropdown option: two space characters per level.",
    },
    "export.option.indent.four": {
        text: "4 spaces",
        description: "Indent dropdown option: four space characters per level.",
    },
    "export.option.indent.tab": {
        text: "Tab",
        description: "Indent dropdown option: one tab character per level.",
    },
    "export.option.delimiter.label": {
        text: "Delimiter",
        description: "Label of the field holding the character that separates cells in a CSV or TSV file.",
    },
    "export.option.delimiter.desc": {
        text: "Character separating cells",
        description: "Caption under the 'Delimiter' field.",
    },
    "export.option.header.label": {
        text: "Include header row",
        description: "Switch label: write the field names as the file's first line.",
    },
    "export.option.header.desc": {
        text: "First line is field names",
        description: "Caption under the 'Include header row' switch.",
    },
    "export.option.bom.label": {
        text: "UTF-8 BOM",
        description: "Switch label: prefix the file with a byte-order mark. 'UTF-8' and 'BOM' are technical terms and stay as-is.",
    },
    "export.option.bom.desc": {
        text: "Helps Excel detect Unicode",
        description: "Caption under the 'UTF-8 BOM' switch. 'Excel' is a product name and stays as-is.",
    },
    "export.option.rowIndex.label": {
        text: "Row index column",
        description: "Switch label: add a leading column numbering the rows.",
    },
    "export.option.rowIndex.desc": {
        text: "Adds a # column before fields",
        description: "Caption under the 'Row index column' switch. The '#' is the literal column heading.",
    },
    "export.option.arrays.label": {
        text: "Arrays as",
        description: "Label of the dropdown choosing how a field holding a list of values is written out. Reads as the start of a sentence the option completes.",
    },
    "export.option.arrays.desc": {
        text: "How list-valued fields are encoded",
        description: "Caption under the 'Arrays as' dropdown.",
    },
    "export.option.arrays.join": {
        text: "Joined string",
        description: "Option: write a list as its values joined by a separator.",
    },
    "export.option.arrays.json": {
        text: "JSON literal",
        description: "Option: write a list as a JSON array.",
    },
    "export.option.arraySep.label": {
        text: "Array separator",
        description: "Label of the field holding the text placed between joined list values.",
    },
    "export.option.arraySep.desc": {
        text: 'Used for "Joined string" mode',
        description: "Caption under the 'Array separator' field. The quoted phrase is the 'Joined string' option above it and should match that translation.",
    },
    "export.option.lineEnding.label": {
        text: "Line endings",
        description: "Label of the dropdown choosing which characters end each line of the file.",
    },
    "export.option.lineEnding.desc": {
        text: "Use CRLF for Windows compatibility",
        description: "Caption under the 'Line endings' dropdown. 'CRLF' and 'Windows' stay as-is.",
    },
    "export.option.lineEnding.lf": {
        text: "LF (Unix)",
        description: "Line-ending option. Both 'LF' and 'Unix' are technical names and stay as-is.",
    },
    "export.option.lineEnding.crlf": {
        text: "CRLF (Windows)",
        description: "Line-ending option. Both 'CRLF' and 'Windows' are technical names and stay as-is.",
    },
    "export.option.xmlRoot.label": {
        text: "Root element",
        description: "Label of the field naming the XML tag that wraps the whole file.",
    },
    "export.option.xmlRoot.desc": {
        text: "Wraps all rows",
        description: "Caption under the 'Root element' field.",
    },
    "export.option.xmlItem.label": {
        text: "Row element",
        description: "Label of the field naming the XML tag used for each row.",
    },
    "export.option.xmlItem.desc": {
        text: "Tag for each row",
        description: "Caption under the 'Row element' field.",
    },
    "export.option.xmlAttrs.label": {
        text: "Primitives as attributes",
        description: "Switch label: write simple values as XML attributes instead of child elements.",
    },
    "export.option.xmlAttrs.desc": {
        text: "Encode strings/numbers as XML attributes",
        description: "Caption under the 'Primitives as attributes' switch.",
    },
    "export.filename.hint": {
        text: "A timestamp is appended automatically.",
        description: "Note under the file-name field in the export dialog.",
    },
    "export.preview.title": {
        text: "Preview",
        description: "Heading of the export dialog's preview pane. A noun.",
    },
    "export.preview.fields": {
        text: "{count, plural, one {# field} other {# fields}}",
        description: "How many columns the export will contain, shown in the preview pane's summary line.",
    },
    "export.preview.empty": {
        text: "Select at least one field to preview the export.",
        description: "Shown in the preview pane when every field checkbox is unticked.",
    },
    "export.preview.truncated": {
        text: "Preview shows first {limit} rows. Full file contains {total} rows.",
        description: "Note under the preview pane when only the first few rows are shown. {limit} is a small whole number and {total} an already-formatted number; both labels are always plural in the source.",
    },
    "export.copy": {
        text: "Copy",
        description: "Button that copies the generated export to the clipboard. A verb.",
    },
    "export.copied": {
        text: "Copied",
        description: "Confirmation the copy button shows for a moment after a successful copy.",
    },
    "export.cancel": {
        text: "Cancel",
        description: "Button that closes the export dialog without exporting.",
    },
    "export.download": {
        text: "Download",
        description: "Primary button of the export dialog; the file size follows it in brackets. A verb.",
    },
} satisfies MessageMap;

// `dynamic`: the format and scope copy is stored in lookup tables and resolved
// as `t(FORMAT_LABELS[fmt].labelKey)`, so the extractor has no literal call
// site to match those keys against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
