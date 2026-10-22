import { defineConfig } from '@doki-land/nifty'

export default defineConfig({
    publish: {
        packages: [
            // Meta-package (CLI shim + optionalDependencies on platforms)
            '@doki-land/dejavu',
            // Platform binaries (os/cpu constrained)
            '@doki-land/dejavu-linux-x64',
            '@doki-land/dejavu-linux-arm64',
            '@doki-land/dejavu-darwin-x64',
            '@doki-land/dejavu-darwin-arm64',
            '@doki-land/dejavu-win32-x64',
            '@doki-land/dejavu-win32-arm64',
        ],
    },
    trust: {
        npm: {
            repo: 'doki-land/dejavu-language',
            file: 'release-npm.yml',
        },
    },
})
