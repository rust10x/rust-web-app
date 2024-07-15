# Development of the Front End in React

## Install js pre-requisites

A working `js` development environment is a prerequisite for `react`. On Ubuntu linux (_or the WSL ubuntu distro on windows_), for instance, this is achieved via `sudo apt install nodejs`. 


## Create the basic project

We use `npm`, the _node package manager_ to install a `vite` based runtime. [vite](https://vitejs.dev/) is a front-end development environment which enables a fast development experience for JS front-ends.
  
  - development http server
  - watch and dynamically re-load: utiles `hmr` (_hot module reload_) to only push deltas to the server
  - integrated build system

`npm create vite@latest` and 
  - `frontend-react` for the name
  - `React`
  - `Typescript` 

Now run `npm install` which will download all the js packages needed (_which are listed in the `package.json` file when the project was created_). 

> When you get a fresh tree from github. There are no cached js libs for building. You'll need to run `npm install` so node can download all the packages listed in `packages.json`.

## Install dependencies

### MaterialUI

We are going to be using MaterialUI based components. Popular and highly documented.

`npm install @mui/icons-material @mui/material @emotion/react @emotion/styled`

### Router DOM

React Apps are `SPA`s (_Single Page Applications_) that start life with a single page push from the server (_a server that serves the react app, usually distinct from the backend API server_). Subsequent changes to the app never requires a page-refresh, they simply need dynamic updates of the DOM via API responses. Even though the app/page state does not directly correspond to server URL paths, a URL-like path still serves a purpose: it provides a name for a logical `app state`. Since we have logical URL's, we also need a way to manage that and for this we need the `react router dom`

`npm install react-router-dom`
