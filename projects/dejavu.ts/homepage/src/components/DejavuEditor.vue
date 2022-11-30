<template>
  <div ref="editorContainer" class="editor-container"></div>
</template>

<script setup lang="ts">
import {ref, onMounted, onBeforeUnmount, watch} from "vue";
import * as monaco from "monaco-editor";
import loader from "@monaco-editor/loader";

loader.config({monaco});

const props = defineProps<{
  modelValue: string;
  language?: string;
  theme?: string;
  readOnly?: boolean;
  height?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

const DEJAVU_LANGUAGE_ID = "dejavu";

function registerDejavuLanguage() {
  monaco.languages.register({id: DEJAVU_LANGUAGE_ID});

  monaco.languages.setMonarchTokensProvider(DEJAVU_LANGUAGE_ID, {
    keywords: [
      "if",
      "else",
      "for",
      "in",
      "while",
      "match",
      "case",
      "break",
      "continue",
      "return",
      "import",
      "from",
      "include",
      "extends",
      "macro",
      "block",
    ],
    typeKeywords: ["string", "int", "float", "bool", "list", "dict", "object", "null"],
    operators: [
      "=",
      "==",
      "!=",
      "<",
      "<=",
      ">",
      ">=",
      "+",
      "-",
      "*",
      "/",
      "%",
      "!",
      "&&",
      "||",
    ],
    symbols: /[=+\-*/%<>!&|]+/,
    escapes: /\\(?:[btnfr\\"']|[0-7]{1,3}|x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})/,
    tokenizer: {
      root: [
        [/\<%.!/, {token: "comment", next: "@comment"}],
        [/\<%[-_~=.]?/, {token: "keyword", next: "@statement"}],
        [/[^<%]+/, "text"],
        [/\{/, "text"],
      ],
      comment: [
        [/.*?%\}/, {token: "comment", next: "@pop"}],
        [/.*/, "comment"],
      ],
      statement: [
        [/%\}/, {token: "keyword", next: "@pop"}],
        [/-?%\}/, {token: "keyword", next: "@pop"}],
        [/_?%\}/, {token: "keyword", next: "@pop"}],
        [/~?%\}/, {token: "keyword", next: "@pop"}],
        [/=?%\}/, {token: "keyword", next: "@pop"}],
        [/\./, {token: "keyword"}],
        [/[-_~=]/, {token: "keyword"}],
        {include: "@whitespace"},
        {include: "@strings"},
        {include: "@numbers"},
        [
          /[a-zA-Z_][\w]*/,
          {
            cases: {
              "@keywords": "keyword",
              "@typeKeywords": "type",
              "@default": "identifier",
            },
          },
        ],
        [
          /@symbols/,
          {
            cases: {
              "@operators": "operator",
              "@default": "",
            },
          },
        ],
        [/[()\[\]{}]/, "@brackets"],
      ],
      whitespace: [
        [/[ \t\r\n]+/, ""],
        [/\/\*/, "comment", "@comment"],
        [/\/\/.*$/, "comment"],
      ],
      strings: [
        [/'/, {token: "string", next: "@stringSingle"}],
        [/"/, {token: "string", next: "@stringDouble"}],
      ],
      stringSingle: [
        [/'/, {token: "string", next: "@pop"}],
        [/@escapes/, "string.escape"],
        [/./, "string"],
      ],
      stringDouble: [
        [/"/, {token: "string", next: "@pop"}],
        [/@escapes/, "string.escape"],
        [/./, "string"],
      ],
      numbers: [
        [/\d+\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/\d+[eE][\-+]?\d+/, "number.float"],
        [/\d+/, "number"],
      ],
    },
  });

  monaco.editor.defineTheme("dejavu-theme", {
    base: "vs",
    inherit: true,
    rules: [
      {token: "keyword", foreground: "0000FF", fontStyle: "bold"},
      {token: "type", foreground: "267F99"},
      {token: "identifier", foreground: "001080"},
      {token: "string", foreground: "A31515"},
      {token: "string.escape", foreground: "FF0000"},
      {token: "number", foreground: "098658"},
      {token: "number.float", foreground: "098658"},
      {token: "operator", foreground: "000000"},
      {token: "comment", foreground: "008000"},
      {token: "brackets", foreground: "000000"},
    ],
    colors: {},
  });

  monaco.editor.defineTheme("dejavu-dark", {
    base: "vs-dark",
    inherit: true,
    rules: [
      {token: "keyword", foreground: "569CD6", fontStyle: "bold"},
      {token: "type", foreground: "4EC9B0"},
      {token: "identifier", foreground: "9CDCFE"},
      {token: "string", foreground: "CE9178"},
      {token: "string.escape", foreground: "D7BA7D"},
      {token: "number", foreground: "B5CEA8"},
      {token: "number.float", foreground: "B5CEA8"},
      {token: "operator", foreground: "D4D4D4"},
      {token: "comment", foreground: "6A9955"},
      {token: "brackets", foreground: "FFD700"},
    ],
    colors: {},
  });
}

function createEditor() {
  if (!editorContainer.value) return;

  registerDejavuLanguage();

  editor = monaco.editor.create(editorContainer.value, {
    value: props.modelValue,
    language: props.language || DEJAVU_LANGUAGE_ID,
    theme: props.theme || "dejavu-theme",
    readOnly: props.readOnly || false,
    automaticLayout: true,
    minimap: {enabled: false},
    scrollBeyondLastLine: false,
    fontSize: 14,
    fontFamily: 'Consolas, "Courier New", monospace',
    lineNumbers: "on",
    renderWhitespace: "selection",
    tabSize: 2,
    insertSpaces: true,
    wordWrap: "on",
  });

  editor.onDidChangeModelContent(() => {
    if (editor) {
      emit("update:modelValue", editor.getValue());
    }
  });
}

function updateEditorValue(newValue: string) {
  if (editor && editor.getValue() !== newValue) {
    editor.setValue(newValue);
  }
}

watch(() => props.modelValue, updateEditorValue);

onMounted(() => {
  createEditor();
});

onBeforeUnmount(() => {
  if (editor) {
    editor.dispose();
  }
});
</script>

<style scoped>
.editor-container {
  width: 100%;
  height: 100%;
  border: 1px solid var(--line);
  border-radius: 0;
  overflow: hidden;
}
</style>
