<template>
  <div class="site-shell">
    <Header :currentSection="currentSection" @sectionChange="navigate"/>

    <main v-if="currentSection === 'home'" class="home-page">
      <section class="hero page-shell">
        <div class="hero-copy">
          <p class="eyebrow"><span class="eyebrow-rule"></span>DEJAVU / TEMPLATE ENGINE</p>
          <h1>One template.<br/><em>Every host.</em></h1>
          <p class="hero-lede">
            A compile-time template language with one shared intermediate representation.
            Write the source once. Keep the render semantics wherever it runs.
          </p>
          <div class="hero-actions">
            <button class="button button-primary" type="button" @click="navigate('playground')">
              Open Playground <span aria-hidden="true">↗</span>
            </button>
            <button class="button button-secondary" type="button" @click="navigate('ecosystem')">
              See the hosts
            </button>
          </div>
          <dl class="hero-facts">
            <div>
              <dt>01</dt>
              <dd>source language</dd>
            </div>
            <div>
              <dt>02</dt>
              <dd>shared IR contract</dd>
            </div>
            <div>
              <dt>03</dt>
              <dd>isomorphic output</dd>
            </div>
          </dl>
        </div>

        <div class="compile-bench">
          <div class="bench-chrome">
            <div class="bench-file"><span class="file-mark">D/</span> greeting.dejavu</div>
            <div class="bench-state"><span class="state-dot"></span>COMPILE PREVIEW</div>
          </div>
          <div class="bench-body">
            <div class="bench-pane source-pane">
              <div class="pane-label"><span class="pane-index">01</span> SOURCE</div>
              <div class="hero-editor">
                <DejavuEditor v-model="heroTemplateCode" theme="dejavu-theme" :readOnly="true"/>
              </div>
            </div>
            <div class="bench-transfer" aria-label="Compile to output">
              <span class="transfer-line"></span>
              <span class="transfer-symbol">→</span>
              <small>render</small>
            </div>
            <div class="bench-pane output-pane">
              <div class="pane-label"><span class="pane-index">02</span> OUTPUT</div>
              <div class="rendered-page">
                <div class="rendered-bar"><span></span><span></span><span></span><small>preview.html</small></div>
                <div class="rendered-content">
                  <span class="output-kicker">DEJAVU / RENDERED</span>
                  <strong>Hello, Mira.</strong>
                  <p>Your items are ready.</p>
                  <ul>
                    <li>Notebook</li>
                    <li>Compass</li>
                    <li>Lantern</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
          <div class="bench-footer"><span>source.dejavu</span><span>→</span><span>Dejavu IR</span><span>→</span><span>render(IR, ctx)</span><b>READY</b>
          </div>
        </div>
      </section>

      <section class="manifesto-band">
        <div class="page-shell manifesto-inner">
          <p>Templates should travel further than the process that created them.</p>
          <span>same source / same meaning</span>
        </div>
      </section>

      <section class="principles page-shell section-block">
        <div class="section-heading">
          <p class="eyebrow"><span class="eyebrow-rule"></span>THE CONTRACT</p>
          <h2>Small surface.<br/><em>Serious guarantees.</em></h2>
          <p>DejaVu keeps the language close to the template author and the IR strict enough for every host.</p>
        </div>
        <div class="principle-list">
          <article v-for="(item, index) in principles" :key="item.title" class="principle-row">
            <span class="row-number">0{{ index + 1 }}</span>
            <h3>{{ item.title }}</h3>
            <p>{{ item.body }}</p>
            <span class="row-token">{{ item.token }}</span>
          </article>
        </div>
      </section>

      <section class="example-section section-block">
        <div class="page-shell">
          <div class="section-heading split-heading">
            <div>
              <p class="eyebrow"><span class="eyebrow-rule"></span>THE SHAPE OF A TEMPLATE</p>
              <h2>Read the source.<br/><em>Then read the result.</em></h2>
            </div>
            <p>Control flow stays visible in the file. The generated output stays boring, predictable HTML.</p>
          </div>
          <div class="example-grid">
            <div class="example-pane">
              <div class="example-label"><span>template.dejavu</span><small>READ-ONLY EXAMPLE</small></div>
              <div class="example-editor">
                <DejavuEditor v-model="getStartedTemplateCode" theme="dejavu-theme" :readOnly="true"/>
              </div>
            </div>
            <div class="example-result">
              <div class="example-label"><span>generated.html</span><small>OUTPUT</small></div>
              <div class="html-result">
                <div class="html-line"><span class="line-no">01</span><span class="tag">&lt;main&gt;</span></div>
                <div class="html-line indent"><span class="line-no">02</span><span class="tag">&lt;h1&gt;</span><span>Hello, Mira.</span><span
                    class="tag">&lt;/h1&gt;</span></div>
                <div class="html-line indent"><span class="line-no">03</span><span class="tag">&lt;p&gt;</span><span>Your items are ready.</span><span
                    class="tag">&lt;/p&gt;</span></div>
                <div class="html-line"><span class="line-no">04</span><span class="tag">&lt;/main&gt;</span></div>
              </div>
              <div class="result-note"><span class="check-mark">✓</span> deterministic output for the same IR + context
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="hosts-section page-shell section-block">
        <div class="section-heading split-heading">
          <div>
            <p class="eyebrow"><span class="eyebrow-rule"></span>HOSTS, NOT FORKS</p>
            <h2>Bring the same<br/><em>template to the stack.</em></h2>
          </div>
          <p>Each host wraps the same public facade. The language can evolve without splitting the semantics.</p>
        </div>
        <div class="host-rail">
          <button v-for="host in hosts" :key="host.id" type="button" class="host-chip" @click="navigate('ecosystem')">
            <span class="host-glyph">{{ host.glyph }}</span>
            <span>{{ host.name }}</span>
            <span class="host-arrow" aria-hidden="true">↗</span>
          </button>
        </div>
        <div class="host-equation"><span>*.dejavu</span><b>→</b><span>Dejavu IR</span><b>→</b><span>host renderer</span>
        </div>
      </section>

      <section class="home-cta page-shell">
        <div><p class="eyebrow"><span class="eyebrow-rule"></span>START WITH THE SOURCE</p>
          <h2>Make the template<br/><em>the portable part.</em></h2></div>
        <button class="button button-primary" type="button" @click="navigate('playground')">Open Playground <span
            aria-hidden="true">↗</span></button>
      </section>
    </main>

    <main v-else-if="currentSection === 'playground'" class="workspace page-shell">
      <header class="workspace-heading">
        <div><p class="eyebrow"><span class="eyebrow-rule"></span>PLAYGROUND / LIVE SURFACE</p>
          <h1>Compile a template.</h1>
          <p>Edit the source, then inspect the output contract.</p></div>
        <span class="workspace-status"><i></i> LOCAL SESSION</span></header>
      <section class="playground-grid">
        <div class="workspace-panel">
          <div class="panel-top"><span>SOURCE / template.dejavu</span><span>EDITABLE</span></div>
          <div class="workspace-editor">
            <DejavuEditor v-model="templateCode" theme="dejavu-theme"/>
          </div>
        </div>
        <div class="workspace-panel">
          <div class="panel-top"><span>OUTPUT / preview</span><span>STATIC PREVIEW</span></div>
          <pre class="workspace-output">{{ outputResult }}</pre>
        </div>
      </section>
      <div class="workspace-actions">
        <button type="button" class="button button-primary" @click="runPlayground">Run template <span
            aria-hidden="true">↗</span></button>
        <button type="button" class="button button-secondary" @click="resetPlayground">Reset source</button>
      </div>
    </main>

    <main v-else class="workspace page-shell ecosystem-page">
      <header class="workspace-heading">
        <div><p class="eyebrow"><span class="eyebrow-rule"></span>ECOSYSTEM / SHARED CONTRACT</p>
          <h1>One IR.<br/><em>Many hosts.</em></h1>
          <p>Host packages wrap the same language and renderer contract. They do not fork the source semantics.</p>
        </div>
        <span class="workspace-status"><i></i> VERSIONED IR</span></header>
      <section class="ecosystem-hosts">
        <article v-for="host in hosts" :key="host.id" class="host-card">
          <div class="host-card-head"><span class="host-glyph">{{ host.glyph }}</span>
            <div><h2>{{ host.name }}</h2>
              <p>{{ host.package }}</p></div>
          </div>
          <pre>{{ host.snippet }}</pre>
          <p v-if="host.note" class="host-note">{{ host.note }}</p></article>
      </section>
      <section class="invariant"><p class="eyebrow"><span class="eyebrow-rule"></span>INVARIANT</p>
        <h2>Same IR + same context = same output.</h2>
        <pre>render_lang(IR, ctx) == expected.out.txt</pre>
      </section>
    </main>

    <Footer @navigate="navigate"/>
  </div>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, ref} from "vue";
import DejavuEditor from "@/components/DejavuEditor.vue";
import Header from "@/components/Header.vue";
import Footer from "@/components/Footer.vue";

type Section = "home" | "playground" | "ecosystem";

const PATHS: Record<Section, string> = {
  home: "/",
  playground: "/playground",
  ecosystem: "/ecosystem",
};

const currentSection = ref<Section>("home");

const principles = [
  {
    title: "Readable source",
    body: "Interpolation and control flow stay close to the text your team edits.",
    token: "<%= value %>",
  },
  {
    title: "Shared IR",
    body: "A versioned intermediate representation is the contract between parser and host.",
    token: "IR / v1",
  },
  {
    title: "Predictable render",
    body: "The same source and context produce the same output, whichever host runs it.",
    token: "render(IR, ctx)",
  },
];

const hosts = [
  {
    id: "cs",
    name: "C# / .NET",
    glyph: ".N",
    package: "Dejavu",
    snippet: "var html = Dj.RenderSource(source, ctx);",
  },
  {
    id: "kt",
    name: "Kotlin",
    glyph: "Kt",
    package: "dejavu",
    snippet: "val html = Dejavu.renderSource(source, ctx)",
  },
  {
    id: "py",
    name: "Python",
    glyph: "Py",
    package: "dejavu (PyPI)",
    snippet: "html = Dejavu.render_source(source, ctx)",
    note: "Scaffold / evolving",
  },
  {
    id: "rs",
    name: "Rust",
    glyph: "Rs",
    package: "dejavu (crates.io)",
    snippet: "let html = Dejavu::render_source(source, ctx);",
    note: "Optional CLI: dejavu-tools",
  },
  {
    id: "ts",
    name: "TypeScript",
    glyph: "TS",
    package: "dejavu (npm)",
    snippet: "const html = Dejavu.renderSource(source, ctx);",
  },
];

const heroTemplateCode = ref(
    `Hello, <%= name %>!\n\n<% if age > 18 %>\n  You are an adult.\n<% else %>\n  You are a minor.\n<% endif %>\n\n<% for item in items %>\n  - <%= item %>\n<% endfor %>`,
);
const getStartedTemplateCode = ref(
    `Hello, <%= name %>!\n<% if age > 18 %>\nYou are an adult.\n<% else %>\nYou are a minor.\n<% endif %>`,
);
const templateCode = ref(getStartedTemplateCode.value);
const outputResult = ref("Click Run to preview output");

function pathToSection(pathname: string): Section {
  const path = pathname.replace(/\/+$/, "") || "/";
  if (path === "/playground") return "playground";
  if (path === "/ecosystem") return "ecosystem";
  return "home";
}

function syncFromLocation() {
  currentSection.value = pathToSection(window.location.pathname);
}

function navigate(section: string) {
  if (!(section in PATHS)) return;
  const next = section as Section;
  currentSection.value = next;
  if (window.location.pathname !== PATHS[next])
    window.history.pushState({section: next}, "", PATHS[next]);
  window.scrollTo({top: 0, behavior: "smooth"});
}

function runPlayground() {
  outputResult.value =
      "Playground runtime wiring is in progress.\n\nTemplate:\n" + templateCode.value;
}

function resetPlayground() {
  templateCode.value = getStartedTemplateCode.value;
  outputResult.value = "Click Run to preview output";
}

onMounted(() => {
  syncFromLocation();
  window.addEventListener("popstate", syncFromLocation);
});
onUnmounted(() => window.removeEventListener("popstate", syncFromLocation));
</script>

<style scoped>
.site-shell {
  min-height: 100vh;
  background: var(--paper);
  color: var(--ink);
}

.page-shell {
  width: min(1180px, calc(100% - 4rem));
  margin-inline: auto;
}

.eyebrow {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  margin: 0;
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.68rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.eyebrow-rule {
  width: 1.8rem;
  height: 1px;
  background: currentColor;
}

h1, h2, h3, p {
  margin-top: 0;
}

button {
  font: inherit;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  min-height: 2.8rem;
  padding: 0 1.15rem;
  border: 1px solid var(--ink);
  border-radius: 0;
  cursor: pointer;
  font-size: 0.83rem;
  font-weight: 700;
  letter-spacing: 0.01em;
  transition: background 140ms ease, color 140ms ease, transform 140ms ease;
}

.button:hover {
  transform: translateY(-2px);
}

.button-primary {
  background: var(--signal);
  border-color: var(--signal);
  color: #fff8ee;
}

.button-primary:hover {
  background: #c94e30;
}

.button-secondary {
  background: transparent;
  color: var(--ink);
}

.button-secondary:hover {
  background: var(--ink);
  color: var(--paper);
}

.hero {
  display: grid;
  grid-template-columns: minmax(0, 0.74fr) minmax(0, 1.26fr);
  gap: clamp(2rem, 6vw, 6rem);
  align-items: center;
  min-height: 43rem;
  padding-block: 5.5rem 5rem;
}

.hero-copy {
  max-width: 31rem;
}

.hero h1 {
  margin: 1.3rem 0 1.25rem;
  color: var(--ink);
  font-size: clamp(3.25rem, 6.4vw, 6rem);
  font-weight: 750;
  letter-spacing: -0.065em;
  line-height: 0.94;
}

.hero h1 em, .section-heading h2 em, .home-cta h2 em, .ecosystem-page h1 em {
  color: var(--signal);
  font-style: normal;
}

.hero-lede {
  max-width: 28rem;
  margin: 0;
  color: var(--muted);
  font-size: 1rem;
  line-height: 1.7;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 2rem;
}

.hero-facts {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.7rem;
  margin: 2.7rem 0 0;
  padding-top: 1rem;
  border-top: 1px solid var(--line);
}

.hero-facts div {
  min-width: 0;
}

.hero-facts dt {
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.7rem;
}

.hero-facts dd {
  margin: 0.35rem 0 0;
  color: var(--muted);
  font-size: 0.72rem;
  line-height: 1.35;
}

.compile-bench {
  border: 1px solid var(--ink);
  background: var(--ink);
  box-shadow: 0.65rem 0.65rem 0 var(--signal);
}

.bench-chrome, .bench-footer, .panel-top, .example-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.bench-chrome {
  min-height: 2.75rem;
  padding: 0 1rem;
  border-bottom: 1px solid #414740;
  color: #d7ddd2;
  font-family: var(--mono);
  font-size: 0.68rem;
}

.bench-file {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.file-mark {
  color: var(--signal);
  font-weight: 700;
}

.bench-state {
  color: #aab6a8;
  font-size: 0.6rem;
  letter-spacing: 0.08em;
}

.state-dot {
  display: inline-block;
  width: 0.4rem;
  height: 0.4rem;
  margin-right: 0.4rem;
  border-radius: 999px;
  background: var(--moss);
}

.bench-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 3.8rem minmax(0, 0.78fr);
  gap: 0.8rem;
  align-items: center;
  padding: 1.1rem;
}

.pane-label {
  margin-bottom: 0.55rem;
  color: #aab6a8;
  font-family: var(--mono);
  font-size: 0.62rem;
  letter-spacing: 0.1em;
}

.pane-index {
  margin-right: 0.35rem;
  color: var(--signal);
}

.hero-editor {
  height: 19rem;
  background: #f8f5ed;
}

.hero-editor :deep(.editor-container) {
  border: 0;
  border-radius: 0;
}

.bench-transfer {
  display: grid;
  justify-items: center;
  gap: 0.35rem;
  color: var(--signal);
  font-family: var(--mono);
}

.transfer-line {
  width: 100%;
  height: 1px;
  background: #596358;
}

.transfer-symbol {
  font-size: 1.5rem;
  line-height: 1;
}

.bench-transfer small {
  color: #9ca79b;
  font-size: 0.6rem;
  text-transform: uppercase;
}

.rendered-page {
  min-height: 19rem;
  border: 1px solid #cfd3c7;
  background: #f8f5ed;
  color: var(--ink);
}

.rendered-bar {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.45rem 0.55rem;
  border-bottom: 1px solid #daddd4;
}

.rendered-bar span {
  width: 0.35rem;
  height: 0.35rem;
  border-radius: 999px;
  background: var(--signal);
}

.rendered-bar span:nth-child(2) {
  background: var(--sun);
}

.rendered-bar span:nth-child(3) {
  background: var(--moss);
}

.rendered-bar small {
  margin-left: auto;
  color: #7a8278;
  font-family: var(--mono);
  font-size: 0.58rem;
}

.rendered-content {
  padding: 1.2rem;
}

.output-kicker {
  display: block;
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.58rem;
  letter-spacing: 0.1em;
}

.rendered-content strong {
  display: block;
  margin: 2rem 0 0.4rem;
  font-size: 1.35rem;
  letter-spacing: -0.03em;
}

.rendered-content p {
  margin: 0;
  color: #687067;
  font-size: 0.8rem;
}

.rendered-content ul {
  margin: 1.3rem 0 0;
  padding: 0.7rem 0 0 1rem;
  border-top: 1px solid #daddd4;
  color: #687067;
  font-family: var(--mono);
  font-size: 0.66rem;
  line-height: 1.8;
}

.bench-footer {
  min-height: 2.2rem;
  padding: 0 1rem;
  border-top: 1px solid #414740;
  color: #9ca79b;
  font-family: var(--mono);
  font-size: 0.61rem;
}

.bench-footer b {
  margin-left: auto;
  color: var(--moss);
  font-weight: 500;
}

.manifesto-band {
  border-block: 1px solid var(--line);
  background: var(--panel);
}

.manifesto-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  padding-block: 1.35rem;
}

.manifesto-inner p {
  margin: 0;
  color: var(--ink);
  font-size: 0.95rem;
  font-weight: 650;
}

.manifesto-inner span {
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.65rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.section-block {
  padding-block: 6.5rem;
}

.section-heading {
  max-width: 36rem;
}

.section-heading h2 {
  margin: 1rem 0;
  color: var(--ink);
  font-size: clamp(2.15rem, 4vw, 4rem);
  font-weight: 720;
  letter-spacing: -0.055em;
  line-height: 0.98;
}

.section-heading > p:last-child {
  margin: 0;
  color: var(--muted);
  line-height: 1.65;
}

.principles {
  display: grid;
  grid-template-columns: minmax(0, 0.75fr) minmax(0, 1.25fr);
  gap: clamp(2.5rem, 8vw, 8rem);
}

.principle-list {
  border-top: 1px solid var(--ink);
}

.principle-row {
  display: grid;
  grid-template-columns: 2.7rem minmax(9rem, 0.75fr) minmax(0, 1.35fr) 7rem;
  gap: 1rem;
  align-items: baseline;
  padding: 1.25rem 0;
  border-bottom: 1px solid var(--line);
}

.row-number, .row-token {
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.66rem;
}

.principle-row h3 {
  margin: 0;
  color: var(--ink);
  font-size: 1rem;
}

.principle-row p {
  margin: 0;
  color: var(--muted);
  font-size: 0.84rem;
  line-height: 1.45;
}

.row-token {
  color: var(--moss);
  text-align: right;
}

.example-section {
  background: var(--ink);
  color: #f7f4eb;
}

.example-section .section-heading h2 {
  color: #f7f4eb;
}

.example-section .section-heading > p:last-child {
  color: #aab6a8;
}

.split-heading {
  display: flex;
  justify-content: space-between;
  gap: 3rem;
  max-width: none;
}

.split-heading > p {
  max-width: 22rem;
  margin-top: 2.2rem !important;
}

.example-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.86fr);
  gap: 1rem;
  margin-top: 3rem;
}

.example-pane, .example-result {
  min-width: 0;
  border: 1px solid #414740;
}

.example-label {
  min-height: 2.45rem;
  padding: 0 0.85rem;
  border-bottom: 1px solid #414740;
  color: #d7ddd2;
  font-family: var(--mono);
  font-size: 0.66rem;
}

.example-label small {
  color: #879286;
  font-size: 0.58rem;
}

.example-editor {
  height: 22rem;
  background: #f8f5ed;
}

.example-editor :deep(.editor-container) {
  border: 0;
  border-radius: 0;
}

.html-result {
  min-height: 18rem;
  padding: 1.3rem;
  color: #d7ddd2;
  font-family: var(--mono);
  font-size: 0.77rem;
  line-height: 2;
}

.html-line {
  display: flex;
  gap: 0.6rem;
}

.html-line.indent {
  padding-left: 1.4rem;
}

.line-no {
  width: 1.5rem;
  color: #687468;
  text-align: right;
}

.tag {
  color: var(--signal);
}

.result-note {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  min-height: 2.5rem;
  padding: 0 0.85rem;
  border-top: 1px solid #414740;
  color: #aab6a8;
  font-size: 0.68rem;
}

.check-mark {
  color: var(--moss);
}

.hosts-section {
  border-bottom: 1px solid var(--line);
}

.host-rail {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 0.6rem;
  margin-top: 3rem;
}

.host-chip {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 0.55rem;
  min-height: 4rem;
  padding: 0.7rem;
  border: 1px solid var(--line);
  border-radius: 0;
  background: transparent;
  color: var(--ink);
  cursor: pointer;
  text-align: left;
}

.host-chip:hover {
  border-color: var(--signal);
  background: var(--panel);
}

.host-glyph, .host-card .host-glyph {
  display: grid;
  place-items: center;
  width: 2rem;
  height: 2rem;
  background: var(--ink);
  color: var(--paper);
  font-family: var(--mono);
  font-size: 0.62rem;
  font-weight: 600;
}

.host-arrow {
  color: var(--signal);
}

.host-equation {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin-top: 1.7rem;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.68rem;
}

.host-equation b {
  color: var(--signal);
  font-weight: 500;
}

.home-cta {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 2rem;
  padding-block: 6rem 7rem;
}

.home-cta h2 {
  margin: 1rem 0 0;
  color: var(--ink);
  font-size: clamp(2.3rem, 4.6vw, 4.6rem);
  font-weight: 720;
  letter-spacing: -0.06em;
  line-height: 0.95;
}

.workspace {
  padding-block: 4.5rem 6rem;
}

.workspace-heading {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 2rem;
  padding-bottom: 2.2rem;
  border-bottom: 1px solid var(--line);
}

.workspace-heading h1 {
  margin: 1rem 0 0.8rem;
  color: var(--ink);
  font-size: clamp(2.5rem, 5vw, 5rem);
  letter-spacing: -0.06em;
  line-height: 0.95;
}

.workspace-heading > div > p:last-child {
  max-width: 34rem;
  margin: 0;
  color: var(--muted);
  line-height: 1.6;
}

.workspace-status {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  color: var(--moss);
  font-family: var(--mono);
  font-size: 0.66rem;
  letter-spacing: 0.08em;
}

.workspace-status i {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--moss);
}

.playground-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 1rem;
  margin-top: 2rem;
}

.workspace-panel {
  border: 1px solid var(--ink);
  background: var(--surface);
}

.panel-top {
  min-height: 2.5rem;
  padding: 0 0.9rem;
  border-bottom: 1px solid var(--line);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.64rem;
  letter-spacing: 0.08em;
}

.workspace-editor {
  height: 31rem;
  background: #f8f5ed;
}

.workspace-editor :deep(.editor-container) {
  border: 0;
  border-radius: 0;
}

.workspace-output {
  min-height: 31rem;
  margin: 0;
  padding: 1.3rem;
  overflow: auto;
  color: var(--ink);
  font-family: var(--mono);
  font-size: 0.78rem;
  line-height: 1.7;
  white-space: pre-wrap;
}

.workspace-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 1rem;
}

.ecosystem-hosts {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.7rem;
  margin-top: 2rem;
}

.host-card {
  padding: 1.1rem;
  border: 1px solid var(--line);
  background: var(--surface);
}

.host-card-head {
  display: flex;
  align-items: center;
  gap: 0.7rem;
}

.host-card h2 {
  margin: 0;
  color: var(--ink);
  font-size: 1.05rem;
}

.host-card-head p {
  margin: 0.2rem 0 0;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.63rem;
}

.host-card pre {
  margin: 1rem 0 0;
  padding: 0.8rem;
  overflow: auto;
  background: var(--ink);
  color: #d7ddd2;
  font-family: var(--mono);
  font-size: 0.68rem;
  line-height: 1.6;
  white-space: pre-wrap;
}

.host-note {
  margin: 0.7rem 0 0;
  color: var(--signal);
  font-size: 0.75rem;
}

.invariant {
  margin-top: 2.5rem;
  padding: 2.2rem;
  border: 1px solid var(--ink);
  background: var(--ink);
  color: #f7f4eb;
}

.invariant h2 {
  margin: 1rem 0 1.5rem;
  font-size: clamp(1.7rem, 3vw, 3rem);
  letter-spacing: -0.04em;
}

.invariant pre {
  margin: 0;
  padding: 1rem;
  border: 1px solid #414740;
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.78rem;
}

@media (max-width: 1040px) {
  .hero {
    grid-template-columns: 1fr;
    gap: 3rem;
    padding-top: 4.5rem;
  }

  .hero-copy {
    max-width: 44rem;
  }

  .compile-bench {
    max-width: 60rem;
  }

  .principles {
    grid-template-columns: 1fr;
    gap: 3rem;
  }

  .host-rail {
    grid-template-columns: repeat(3, 1fr);
  }

  .host-chip:last-child {
    grid-column: 2;
  }

  .ecosystem-hosts {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 720px) {
  .page-shell {
    width: min(100% - 2rem, 42rem);
  }

  .hero {
    min-height: auto;
    padding-block: 3.5rem 4rem;
  }

  .hero h1 {
    font-size: clamp(3rem, 14vw, 4.5rem);
  }

  .hero-facts {
    gap: 0.5rem;
  }

  .hero-facts dd {
    font-size: 0.67rem;
  }

  .bench-body {
    grid-template-columns: 1fr;
    gap: 1.1rem;
    padding: 0.75rem;
  }

  .bench-transfer {
    grid-template-columns: 1fr auto 1fr;
    display: grid;
  }

  .transfer-line {
    width: 100%;
  }

  .transfer-symbol {
    transform: rotate(90deg);
  }

  .rendered-page, .hero-editor {
    min-height: 0;
    height: 16rem;
  }

  .bench-footer {
    flex-wrap: wrap;
    padding-block: 0.55rem;
  }

  .bench-footer b {
    margin-left: 0;
  }

  .manifesto-inner, .home-cta {
    align-items: flex-start;
    flex-direction: column;
  }

  .section-block {
    padding-block: 4.5rem;
  }

  .split-heading {
    display: block;
  }

  .split-heading > p {
    margin-top: 1.4rem !important;
  }

  .principle-row {
    grid-template-columns: 2.2rem 1fr;
    gap: 0.4rem 0.8rem;
  }

  .principle-row p {
    grid-column: 2;
  }

  .row-token {
    grid-column: 2;
    text-align: left;
  }

  .example-grid, .playground-grid {
    grid-template-columns: 1fr;
  }

  .example-editor {
    height: 18rem;
  }

  .host-rail {
    grid-template-columns: repeat(2, 1fr);
  }

  .host-chip:last-child {
    grid-column: auto;
  }

  .host-equation {
    flex-wrap: wrap;
    gap: 0.55rem;
  }

  .workspace {
    padding-block: 3rem 4rem;
  }

  .workspace-heading {
    align-items: flex-start;
    flex-direction: column;
  }

  .workspace-heading h1 {
    font-size: clamp(2.7rem, 13vw, 4rem);
  }

  .workspace-editor, .workspace-output {
    min-height: 23rem;
    height: 23rem;
  }

  .ecosystem-hosts {
    grid-template-columns: 1fr;
  }

  .invariant {
    padding: 1.2rem;
  }
}

@media (prefers-reduced-motion: reduce) {
  .button, .host-chip {
    transition: none;
  }
}
</style>
