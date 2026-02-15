"""Custom loss functions for handling class imbalance."""

from __future__ import annotations

import torch
import torch.nn as nn
import torch.nn.functional as F


class FocalLoss(nn.Module):
    """Focal Loss for handling class imbalance.

    Focal loss down-weights easy examples and focuses training on hard examples.
    Proposed in "Focal Loss for Dense Object Detection" (Lin et al., 2017).

    Args:
        alpha: Class weights (tensor of shape [num_classes]) or None
        gamma: Focusing parameter (default 2.0). Higher gamma = more focus on hard examples
        reduction: 'mean' or 'sum' or 'none'
    """

    def __init__(
        self,
        alpha: torch.Tensor | None = None,
        gamma: float = 2.0,
        reduction: str = "mean",
    ):
        super().__init__()
        self.alpha = alpha
        self.gamma = gamma
        self.reduction = reduction

    def forward(self, inputs: torch.Tensor, targets: torch.Tensor) -> torch.Tensor:
        """Compute focal loss.

        Args:
            inputs: Logits of shape [batch_size, num_classes]
            targets: Class indices of shape [batch_size]

        Returns:
            Focal loss (scalar if reduction='mean' or 'sum')
        """
        # Get probabilities
        p = F.softmax(inputs, dim=1)

        # Get class probabilities for the true class
        ce_loss = F.cross_entropy(inputs, targets, reduction="none")
        p_t = p.gather(1, targets.view(-1, 1)).squeeze(1)

        # Compute focal term: (1 - p_t)^gamma
        focal_weight = (1 - p_t) ** self.gamma

        # Apply focal weight
        loss = focal_weight * ce_loss

        # Apply alpha weighting if provided
        if self.alpha is not None:
            alpha_t = self.alpha.gather(0, targets)
            loss = alpha_t * loss

        # Apply reduction
        if self.reduction == "mean":
            return loss.mean()
        elif self.reduction == "sum":
            return loss.sum()
        else:
            return loss
