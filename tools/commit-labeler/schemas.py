"""Shared Pydantic schemas for commit classification."""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class CommitLabel(str, Enum):
    feat = "feat"
    fix = "fix"
    refactor = "refactor"
    chore = "chore"
    docs = "docs"
    test = "test"
    perf = "perf"
    style = "style"
    ci = "ci"
    build = "build"
    i18n = "i18n"
    revert = "revert"
    other = "other"


class LabelEntry(BaseModel):
    """A single label with confidence."""

    label: CommitLabel = Field(description="The commit type category")
    confidence: float = Field(ge=0.0, le=1.0, description="How confident")


class CommitClassification(BaseModel):
    """Multi-label classification for one commit."""

    labels: list[LabelEntry] = Field(
        description="Up to 3 labels, most confident first", max_length=3
    )
    reasoning: str = Field(description="One sentence explanation")


class BatchClassification(BaseModel):
    """Classification results for a batch of commits."""

    classifications: list[CommitClassification] = Field(
        description="One classification per commit, in the same order as input"
    )
