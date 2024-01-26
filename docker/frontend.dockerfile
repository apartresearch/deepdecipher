FROM node:18.19-alpine3.19 as frontend-build
WORKDIR /deepdecipher
COPY ./frontend/package.json ./frontend/package-lock.json ./
RUN npm install --frozen-lockfile

COPY ./frontend/ .
RUN npm run build

FROM node:18.19-alpine3.19 as frontend-prod
WORKDIR /deepdecipher
COPY --from=frontend-build /deepdecipher/ ./

CMD PUBLIC_BACKEND_ORIGIN=${BACKEND_ORIGIN} PUBLIC_BACKEND_PORT=${BACKEND_PORT} node build