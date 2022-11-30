<template>
  <div class="shiki-code-wrapper">
    <pre class="shiki-container"><code>{{ code }}</code></pre>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  code: string;
  lang: string;
  theme?: string;
}>();
</script>

<style scoped>
.shiki-code-wrapper {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.shiki-container {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: auto;
  padding: 1rem;
  border: 1px solid var(--line);
  border-radius: 0;
  background: var(--ink);
  color: #d7ddd2;
  font-family: var(--mono);
  font-size: 0.8rem;
  line-height: 1.7;
  white-space: pre-wrap;
}

</style>
