import { Module } from "@nestjs/common";
import { NestFactory } from "@nestjs/core";
import {
  ExpressAdapter,
  NestExpressApplication,
} from "@nestjs/platform-express";
import { PrismaClient } from "@prisma/client";
import path from "path";
import "reflect-metadata";
import { TypeGraphQLModule } from "typegraphql-nestjs";
import {
  FindFirstMetaResolver,
  FindManyBridgeTransferResolver,
  FindManyProofResolver,
  FindUniqueBridgeTransferResolver,
  FindUniqueProofResolver,
} from "../prisma/generated/type-graphql";

interface Context {
  prisma: PrismaClient;
}

async function main() {
  const prisma = new PrismaClient();

  @Module({
    imports: [
      TypeGraphQLModule.forRoot({
        debug: false,
        playground: true,
        // TODO: disable on prod
        introspection: true,
        path: "/",
        emitSchemaFile: path.resolve(__dirname, "./generated-schema.graphql"),
        validate: false,
        context: (): Context => ({ prisma }),
      }),
    ],
    providers: [
      FindFirstMetaResolver,

      FindUniqueBridgeTransferResolver,
      FindManyBridgeTransferResolver,

      FindManyProofResolver,
      FindUniqueProofResolver,
    ],
  })
  class AppModule {}

  const app = await NestFactory.create<NestExpressApplication>(
    AppModule,
    new ExpressAdapter(),
  );

  await app.listen(4000);
  console.log(`GraphQL is listening on 4000!`);
}

main().catch(console.error);
