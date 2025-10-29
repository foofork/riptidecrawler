#!/usr/bin/env python3
"""
Session Management Example

Demonstrates how to use the Sessions API for managing browser sessions,
cookies, and persistent user data.
"""

import asyncio
from riptide_sdk import RipTideClient, SessionConfig, SetCookieRequest


async def main():
    """Main example function demonstrating session management"""

    async with RipTideClient(base_url="http://localhost:8080") as client:
        print("=== RipTide Sessions API Example ===\n")

        # 1. Create a new session
        print("1. Creating a new session...")
        config = SessionConfig(ttl_seconds=3600)  # 1 hour TTL
        session = await client.sessions.create(config=config)
        print(f"   Session ID: {session.session_id}")
        print(f"   User Data Dir: {session.user_data_dir}")
        print(f"   Created At: {session.created_at}")
        print(f"   Expires At: {session.expires_at}\n")

        # 2. Set cookies for the session
        print("2. Setting cookies for the session...")
        cookie1 = SetCookieRequest(
            domain="example.com",
            name="session_token",
            value="abc123xyz",
            path="/",
            expires_in_seconds=3600,
            secure=True,
            http_only=True,
        )
        await client.sessions.set_cookie(session.session_id, cookie1)
        print("   Set session_token cookie for example.com\n")

        cookie2 = SetCookieRequest(
            domain="example.com",
            name="user_preference",
            value="dark_mode",
            path="/",
        )
        await client.sessions.set_cookie(session.session_id, cookie2)
        print("   Set user_preference cookie for example.com\n")

        # 3. Get session information
        print("3. Getting session information...")
        session_info = await client.sessions.get(session.session_id)
        print(f"   Session ID: {session_info.session_id}")
        print(f"   Cookie Count: {session_info.cookie_count}")
        print(f"   Total Domains: {session_info.total_domains}\n")

        # 4. Retrieve cookies for a domain
        print("4. Retrieving cookies for example.com...")
        cookies = await client.sessions.get_cookies(session.session_id, "example.com")
        for cookie in cookies:
            print(f"   - {cookie.name}: {cookie.value}")
            print(f"     Domain: {cookie.domain}")
            print(f"     Path: {cookie.path}")
            print(f"     Secure: {cookie.secure}")
            print(f"     HttpOnly: {cookie.http_only}\n")

        # 5. List all sessions
        print("5. Listing all active sessions...")
        sessions = await client.sessions.list(limit=10)
        print(f"   Total active sessions: {len(sessions)}")
        for sid in sessions[:3]:  # Show first 3
            print(f"   - {sid}")
        print()

        # 6. Extend session expiry
        print("6. Extending session expiry by 1 hour...")
        await client.sessions.extend(session.session_id, 3600)
        print("   Session expiry extended\n")

        # 7. Get session statistics
        print("7. Getting session statistics...")
        stats = await client.sessions.get_stats()
        print(f"   Total sessions: {stats.total_sessions}")
        print(f"   Expired sessions cleaned: {stats.expired_sessions_cleaned}\n")

        # 8. Delete the session
        print("8. Deleting the session...")
        await client.sessions.delete(session.session_id)
        print("   Session deleted successfully\n")

        print("=== Example Complete ===")


if __name__ == "__main__":
    asyncio.run(main())
