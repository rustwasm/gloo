// @ts-check
// `@type` JSDoc annotations allow editor autocompletion and type checking
// (when paired with `@ts-check`).
// There are various equivalent ways to declare your Docusaurus config.
// See: https://docusaurus.io/docs/api/docusaurus-config

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Gloo',
  tagline: ' A modular toolkit for building fast, reliable Web applications and libraries with Rust and Wasm ',
  url: 'https://gloo-rs.web.app',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  favicon: 'img/Gloo-Logo.ico',
  organizationName: 'rustwasm',
  projectName: 'gloo',
  themeConfig: {
    navbar: {
      title: 'Gloo',
      logo: {
        alt: 'Ferris <3 Gloo',
        src: 'img/Gloo-Logo.svg',
      },
      items: [
        {to: '/blog', label: 'Blog', position: 'left'},
        {
          type: 'dropdown',
          label: 'docs.rs',
          position: 'right',
          items: [
            {
              label: 'gloo',
              href: 'https://docs.rs/gloo',
            },
            {
              label: 'dialogs',
              href: 'https://docs.rs/gloo-dialogs/',
            },
            {
              label: 'events',
              href: 'https://docs.rs/gloo-events/',
            },
            {
              label: 'file',
              href: 'https://docs.rs/gloo-file/',
            },
            {
              label: 'history',
              href: 'https://docs.rs/gloo-history/',
            },
            {
              label: 'net',
              href: 'https://docs.rs/gloo-net/',
            },
            {
              label: 'render',
              href: 'https://docs.rs/gloo-render/',
            },
            {
              label: 'storage',
              href: 'https://docs.rs/gloo-storage/',
            },
            {
              label: 'timers',
              href: 'https://docs.rs/gloo-timers/',
            },
            {
              label: 'utils',
              href: 'https://docs.rs/gloo-utils/',
            },
            {
              label: 'worker',
              href: 'https://docs.rs/gloo-worker/',
            }
          ],
        },
        {
          href: 'https://github.com/rustwasm/gloo',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discord.gg/DFgBACbDVG', // #gloo in Yew server
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/rustwasm/gloo',
            },
          ],
        },
      ],
    },
    prism: {
      theme: lightCodeTheme,
      darkTheme: darkCodeTheme,
      additionalLanguages: ['rust', 'toml'],
    },
  },
  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          editUrl:
            'https://github.com/rustwasm/gloo/blob/master/website/',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          editUrl:
            'https://github.com/rustwasm/gloo/blob/master/website/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],
};

module.exports = config;
