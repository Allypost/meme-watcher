import { type Metadata, type Viewport } from "next";

import { env } from "~/env.mjs";

const siteName = "Meme Watcher";

const titleTemplate = {
  template: "%s | Meme Watcher",
  default: "Meme Watcher",
};

const description =
  "Watches memes from a folder and does some processing to make them easier to sort";

export const BASE_VIEWPORT = {
  colorScheme: "dark",
  themeColor: "#000000",
  width: "device-width",
  initialScale: 1,
} satisfies Viewport;

export const BASE_METADATA = {
  metadataBase: new URL(env.NEXT_PUBLIC_APP_URL),
  title: titleTemplate,
  description,
  openGraph: {
    type: "website",
    locale: "hr_HR",
    url: "/",
    siteName,
    countryName: "Croatia",
    title: titleTemplate,
    description,
  },
  twitter: {
    card: "summary_large_image",
    title: titleTemplate,
    description,
  },
  alternates: {
    canonical: "/",
  },
  applicationName: siteName,
  referrer: "origin-when-cross-origin",
  robots: {
    index: true,
    follow: true,
    nocache: false,
    nosnippet: false,
    noimageindex: false,
  },
  icons: {
    icon: "/favicon.ico",
  },
  appleWebApp: {
    statusBarStyle: "black-translucent",
  },
} satisfies Metadata;

export type TemplateString = Exclude<NonNullable<Metadata["title"]>, string>;

export type MetaImageObj = {
  url: string | URL;
  secureUrl?: string | URL;
  alt?: string;
  type?: string;
  width?: string | number;
  height?: string | number;
};

export type MetadataImage = string | MetaImageObj | URL;

export type BasicMetadata = {
  title: string;
  description?: string;
  image?: MetadataImage | MetadataImage[];
};

export const $metadata = (metadata: BasicMetadata): Metadata => {
  const base = {
    ...BASE_METADATA,
    title: {
      ...BASE_METADATA.title,
    },
    openGraph: {
      ...BASE_METADATA.openGraph,
    },
    twitter: {
      ...BASE_METADATA.twitter,
    },
  } as Metadata;

  if (metadata.title) {
    base.title = metadata.title;
    base.openGraph!.title = metadata.title;
    base.twitter!.title = metadata.title;
  }

  if (metadata.description) {
    base.description = metadata.description;
    base.openGraph!.description = metadata.description;
    base.twitter!.description = metadata.description;
  }

  if (metadata.image) {
    base.openGraph!.images = metadata.image;
    base.twitter!.images = metadata.image;
  }

  return {
    ...BASE_METADATA,
    title: {
      ...BASE_METADATA.title,
      default: metadata.title,
    },
    description: metadata.description ?? BASE_METADATA.description,
    openGraph: {
      ...BASE_METADATA.openGraph,
      title: metadata.title,
      description: metadata.description ?? BASE_METADATA.description,
    },
    twitter: {
      ...BASE_METADATA.twitter,
      title: metadata.title,
      description: metadata.description ?? BASE_METADATA.description,
    },
  };
};
