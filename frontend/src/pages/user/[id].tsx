"use client"

import type { GetServerSideProps } from "next"
import type { CharacterData, StoredUser } from "~/types/api"

interface UserPageProps {
  userData: StoredUser | null
  error?: string
}

export default function UserPage({ userData, error }: UserPageProps) {
    return (
        <>
            <p>Hello world</p>
        </>
    )
}

// Helper function to get the correct operator image URL
function getOperatorImageUrl(character: CharacterData): string {
  const { charId, skin, evolvePhase, currentTmpl, tmpl } = character

  // Check if using a custom skin
  // The skin field contains the current equipped skin ID (e.g., "char_002_amiya@epoque#4")
  // Default skins end with #1 (e.g., "char_002_amiya#1")
  const isDefaultSkin = !skin || skin.endsWith("#1") || skin === charId

  if (!isDefaultSkin && skin) {
    // Custom skin - use skinpack
    // Convert skin ID to file path: replace @ with _ and encode # as %23
    const skinFileName = skin.replace(/@/g, "_").replace(/#/g, "%23")
    return `/api/cdn/upk/skinpack/${charId}/${skinFileName}.png`
  }

  // Check template skin if available
  if (currentTmpl && tmpl && tmpl[currentTmpl]) {
    const templateSkin = tmpl[currentTmpl].skinId
    if (templateSkin && !templateSkin.endsWith("#1")) {
      const skinFileName = templateSkin.replace(/@/g, "_").replace(/#/g, "%23")
      return `/api/cdn/upk/skinpack/${charId}/${skinFileName}.png`
    }
  }

  // Default skin - use chararts with evolve phase
  // E2 uses _2.png, E0/E1 uses _1.png
  const suffix = evolvePhase >= 2 ? "_2" : "_1"
  return `/api/cdn/upk/chararts/${charId}/${charId}${suffix}.png`
}

// Server-side data fetching
export const getServerSideProps: GetServerSideProps<UserPageProps> = async (context) => {
  const { id } = context.params as { id: string }

  try {
    // Dynamically import env to avoid issues with server-side rendering
    const { env } = await import("~/env")

    // Build backend URL to get user data
    const backendUrl = new URL("/get-user", env.BACKEND_URL)
    backendUrl.searchParams.set("uid", id)

    // Fetch user data from backend
    const response = await fetch(backendUrl.toString(), {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    })

    if (!response.ok) {
      if (response.status === 404) {
        return {
          props: {
            userData: null,
            error: "User not found",
          },
        }
      }
      const errorText = await response.text()
      console.error(`Backend get-user failed: ${response.status} - ${errorText}`)
      return {
        props: {
          userData: null,
          error: "Failed to fetch user data",
        },
      }
    }

    // Backend returns StoredUser: { id, uid, server, data, created_at }
    const userData: StoredUser = await response.json()

    if (!userData || !userData.data || !userData.data.status) {
      return {
        props: {
          userData: null,
          error: "Invalid user data received",
        },
      }
    }

    return {
      props: {
        userData,
      },
    }
  } catch (error) {
    console.error("Error fetching user data:", error)
    return {
      props: {
        userData: null,
        error: error instanceof Error ? error.message : "Failed to fetch user data",
      },
    }
  }
}
