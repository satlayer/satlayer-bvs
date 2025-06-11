/**
 * @see https://prettier.io/docs/configuration
 * @type {import("prettier").Config}
 */
const config = {
  printWidth: 120,
  plugins: ["prettier-plugin-tailwindcss", "prettier-plugin-packagejson", "prettier-plugin-toml"],
};

module.exports = config;
