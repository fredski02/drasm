// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-04-03",
  devtools: { enabled: true },
  modules: ["@nuxtjs/tailwindcss"],

  runtimeConfig: {
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || "http://localhost:3000/api",
    },
  },

  app: {
    head: {
      title: "DRASM - Distributed WASM Execution",
      meta: [
        { charset: "utf-8" },
        { name: "viewport", content: "width=device-width, initial-scale=1" },
        {
          name: "description",
          content: "DRASM - Distributed WASM execution platform",
        },
      ],
    },
  },

  typescript: {
    strict: true,
    typeCheck: false, // Disable type checking to avoid requiring vue-tsc
  },
});
