<template>
  <footer class="site-footer">
    <div class="footer-inner">
      <div class="footer-brand"><span class="footer-mark">D/</span>
        <div><strong>DejaVu</strong>
          <p>Portable templates on a shared IR.</p></div>
      </div>
      <div class="footer-nav">
        <div><span>EXPLORE</span>
          <button type="button" @click="$emit('navigate', 'playground')">Playground</button>
          <button type="button" @click="$emit('navigate', 'ecosystem')">Ecosystem</button>
        </div>
        <div><span>HOSTS</span>
          <p>TypeScript · Rust · C#</p>
          <p>Kotlin · Python</p></div>
      </div>
    </div>
    <div class="footer-bar"><span>© {{ year }} DejaVu</span><span class="footer-code">source → IR → output</span></div>
  </footer>
</template>

<script setup lang="ts">
defineEmits<{ navigate: [section: string] }>();
const year = new Date().getFullYear();
</script>

<style scoped>
.site-footer {
  border-top: 1px solid #3c423c;
  background: var(--ink);
  color: #d7ddd2;
}

.footer-inner {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.8fr);
  gap: 4rem;
  width: min(1180px, calc(100% - 4rem));
  margin-inline: auto;
  padding-block: 3.5rem 2.5rem;
}

.footer-brand {
  display: flex;
  gap: 0.85rem;
  align-items: flex-start;
}

.footer-mark {
  display: grid;
  place-items: center;
  width: 2.6rem;
  height: 2.6rem;
  background: var(--signal);
  color: #fff8ee;
  font-family: var(--mono);
  font-size: 0.78rem;
  font-weight: 700;
}

.footer-brand strong {
  color: #f6f3eb;
  font-size: 1.1rem;
}

.footer-brand p {
  margin: 0.35rem 0 0;
  color: #9ca79b;
  font-size: 0.82rem;
}

.footer-nav {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
}

.footer-nav > div {
  display: grid;
  align-content: start;
  justify-items: start;
  gap: 0.4rem;
}

.footer-nav span {
  margin-bottom: 0.35rem;
  color: var(--signal);
  font-family: var(--mono);
  font-size: 0.61rem;
  letter-spacing: 0.12em;
}

.footer-nav button {
  padding: 0;
  border: 0;
  background: transparent;
  color: #aab6a8;
  cursor: pointer;
  font-size: 0.8rem;
}

.footer-nav button:hover {
  color: #f6f3eb;
}

.footer-nav p {
  margin: 0;
  color: #8c978b;
  font-size: 0.75rem;
}

.footer-bar {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  width: min(1180px, calc(100% - 4rem));
  margin-inline: auto;
  padding-block: 1rem 1.25rem;
  border-top: 1px solid #3c423c;
  color: #788277;
  font-size: 0.7rem;
}

.footer-code {
  font-family: var(--mono);
  font-size: 0.62rem;
}

@media (max-width: 720px) {
  .footer-inner {
    grid-template-columns: 1fr;
    gap: 2.5rem;
    width: min(100% - 2rem, 42rem);
  }

  .footer-bar {
    flex-direction: column;
    width: min(100% - 2rem, 42rem);
  }

  .footer-nav {
    gap: 1rem;
  }
}
</style>
