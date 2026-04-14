# Author: Quadri Atharu
"""JWT authentication middleware for the HAQLY AI Finance Engine."""

from __future__ import annotations

import jwt
from datetime import datetime, timedelta, timezone
from typing import Any, Dict, Optional

from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer

SECRET_KEY = "haqly_ai_finance_engine_secret_key_change_in_production"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 60

oauth2_scheme = OAuth2PasswordBearer(url="/api/auth/token")

SKIP_AUTH_PATHS = {"/health", "/docs", "/openapi.json", "/redoc"}


def verify_token(token: str, secret: str = SECRET_KEY) -> Dict[str, Any]:
    """Verify and decode a JWT token, returning the claims.

    Args:
        token: The JWT token string.
        secret: The secret key used for verification.

    Returns:
        The decoded token claims as a dict.

    Raises:
        HTTPException: If the token is invalid, expired, or missing required claims.
    """
    try:
        payload = jwt.decode(token, secret, algorithms=[ALGORITHM])
        if "sub" not in payload:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Token missing required 'sub' claim",
                headers={"WWW-Authenticate": "Bearer"},
            )
        return payload
    except jwt.ExpiredSignatureError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token has expired",
            headers={"WWW-Authenticate": "Bearer"},
        )
    except jwt.InvalidTokenError as exc:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Invalid token: {exc}",
            headers={"WWW-Authenticate": "Bearer"},
        )


def create_access_token(
    data: Dict[str, Any],
    expires_delta: Optional[timedelta] = None,
    secret: str = SECRET_KEY,
) -> str:
    """Create a signed JWT access token.

    Args:
        data: Payload to encode into the token.
        expires_delta: Custom expiry duration. Defaults to ACCESS_TOKEN_EXPIRE_MINUTES.
        secret: Secret key for signing.

    Returns:
        Encoded JWT token string.
    """
    to_encode = data.copy()
    expire = datetime.now(timezone.utc) + (expires_delta or timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES))
    to_encode.update({"exp": expire, "iat": datetime.now(timezone.utc)})
    return jwt.encode(to_encode, secret, algorithm=ALGORITHM)


async def get_current_user(token: str = Depends(oauth2_scheme)) -> Dict[str, Any]:
    """FastAPI dependency that extracts and verifies the current user from a Bearer token.

    Skips authentication for paths in SKIP_AUTH_PATHS.

    Returns:
        User dict with at least 'sub' (user ID), 'role', and 'company_id'.

    Raises:
        HTTPException: 401 if token is invalid or expired.
    """
    claims = verify_token(token)
    return {
        "user_id": claims.get("sub", ""),
        "role": claims.get("role", "user"),
        "company_id": claims.get("company_id", ""),
        "email": claims.get("email", ""),
        "exp": claims.get("exp"),
    }
