# Frontend

This is the frontend for the Meme Watcher app.

For now it just displays data provided by the backend.

It's built with NextJS, TypeScript and TailwindCSS.
Types for the backend are automatically generated _by_ the backend.

## Requirements

A relatively modern [NodeJS runtime](https://nodejs.org/) (20+), and [pnpm](https://pnpm.io/).

## Running

Before you run any commands, you should **copy the `.env.example` file into `.env`**.
Edit the `.env` file with the correct values. All options should be documented in comments inside the file.

To run the application, there are three steps:

1. `pnpm install` to install the dependencies
2. `pnpm run build` to build the application
3. `pnpm run start` to run the application

## Development

For development, eslint and prettier are used to check and format the code.
Eslint is set up to also report the prettier issues.

Run `pnpm run dev` to start the development server.

To format the code use `pnpm run lint` and add the `--fix` flag to auto-fix issues.

Do not edit files in the `src/types/generated` directory. Those are generated automatically and should _never_ be edited manually.
