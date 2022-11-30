<template>
  <header class="site-header">
    <div class="header-inner">
      <button class="brand" type="button" aria-label="DejaVu home" @click="$emit('sectionChange', 'home')">
        <span class="brand-mark">D/</span>
        <span class="brand-name">DejaVu</span>
        <span class="brand-caption">compile-time templates</span>
      </button>
      <nav aria-label="Primary navigation">
        <button v-for="item in items" :key="item.id" type="button" :data-active="currentSection === item.id"
                @click="$emit('sectionChange', item.id)">
          <span>{{ item.index }}</span>{{ item.label }}
        </button>
      </nav>
      <button class="header-cta" type="button" @click="$emit('sectionChange', 'playground')">Open Playground <span
          aria-hidden="true">↗</span></button>
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{ currentSection: string }>();
defineEmits<{ sectionChange: [section: string] }>();

const items = [
  {id: "home", index: "01", label: "Home"},
  {id: "playground", index: "02", label: "Playground"},
  {id: "ecosystem", index: "03", label: "Ecosystem"},
];
</script>

<style scoped>
.site-header {
  position: sticky;
  top: 0;
  z-index: 50;
  border-bottom: 1px solid var(--line);
  background: rgba(244, 240, 230, 0.94);
  backdrop-filter: blur(12px);
}

.header-inner {
  display: grid;
  grid-template-columns: minmax(16rem, 1fr) auto minmax(16rem, 1fr);
  align-items: center;
  gap: 1rem;
  width: min(1180px, calc(100% - 4rem));
  min-height: 4.35rem;
  margin-inline: auto;
}

button {
  border: 0;
  font: inherit;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  min-width: 0;
  padding: 0;
  background: transparent;
  color: var(--ink);
  cursor: pointer;
  text-align: left;
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 2.15rem;
  height: 2.15rem;
  background: var(--ink);
  color: var(--paper);
  font-family: var(--mono);
  font-size: 0.72rem;
  font-weight: 700;
}

.brand-name {
  font-size: 1rem;
  font-weight: 750;
  letter-spacing: -0.02em;
}

.brand-caption {
  padding-left: 0.7rem;
  border-left: 1px solid var(--line);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.6rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

nav {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

nav button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  min-height: 2.35rem;
  padding: 0 0.7rem;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  font-size: 0.77rem;
  font-weight: 650;
}

nav button span {
  color: #a99f91;
  font-family: var(--mono);
  font-size: 0.56rem;
}

nav button:hover, nav button[data-active="true"] {
  color: var(--ink);
}

nav button[data-active="true"] {
  box-shadow: inset 0 -2px var(--signal);
}

nav button[data-active="true"] span {
  color: var(--signal);
}

.header-cta {
  justify-self: end;
  min-height: 2.35rem;
  padding: 0 0.8rem;
  border: 1px solid var(--signal);
  background: var(--signal);
  color: #fff8ee;
  cursor: pointer;
  font-size: 0.72rem;
  font-weight: 700;
}

.header-cta:hover {
  background: #c94e30;
}

@media (max-width: 900px) {
  .header-inner {
    grid-template-columns: auto 1fr auto;
  }

  .brand-caption {
    display: none;
  }
}

@media (max-width: 680px) {
  .header-inner {
    grid-template-columns: 1fr auto;
    width: min(100% - 2rem, 42rem);
    padding-block: 0.65rem;
  }

  .brand {
    grid-column: 1;
    grid-row: 1;
  }

  .brand-name {
    font-size: 0.92rem;
  }

  .header-cta {
    grid-column: 2;
    grid-row: 1;
    padding-inline: 0.65rem;
  }

  .header-cta > span {
    display: none;
  }

  nav {
    grid-column: 1 / -1;
    grid-row: 2;
    justify-content: flex-start;
    overflow-x: auto;
  }

  nav button {
    padding-inline: 0.5rem;
    white-space: nowrap;
  }
}
</style>
