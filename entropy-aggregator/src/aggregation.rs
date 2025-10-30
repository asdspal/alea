use std::collections::{HashMap, BTreeMap};
use entropy_types::{NodeId, RevealPayload};

/// Sorts reveals by NodeId lexicographically and concatenates the secrets in that order
/// 
/// This function ensures deterministic ordering of secrets by using BTreeMap which
/// automatically sorts keys (NodeIds) lexicographically.
/// 
/// # Arguments
/// * `reveals` - A HashMap mapping NodeIds to RevealPayloads containing secrets
/// 
/// # Returns
/// * `Vec<u8>` - Concatenated bytes of all secrets in lexicographically sorted order of NodeIds
pub fn sort_and_concatenate_secrets(reveals: HashMap<NodeId, RevealPayload>) -> Vec<u8> {
    // Use BTreeMap to automatically sort NodeIds lexicographically
    let mut sorted_map: BTreeMap<NodeId, RevealPayload> = BTreeMap::new();
    
    // Insert all reveals into the BTreeMap, which will sort them by NodeId
    for (node_id, reveal_payload) in reveals {
        sorted_map.insert(node_id, reveal_payload);
    }
    
    // Concatenate the secrets in the sorted order
    let mut concatenated_secrets = Vec::new();
    for (_, reveal_payload) in sorted_map {
        concatenated_secrets.extend_from_slice(&reveal_payload.secret);
    }
    
    concatenated_secrets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_sorting() {
        // Create a set of reveals with different NodeIds
        let mut reveals = HashMap::new();
        reveals.insert("node3".to_string(), RevealPayload {
            round_id: 1,
            secret: [3u8; 32],
        });
        reveals.insert("node1".to_string(), RevealPayload {
            round_id: 1,
            secret: [1u8; 32],
        });
        reveals.insert("node2".to_string(), RevealPayload {
            round_id: 1,
            secret: [2u8; 32],
        });

        // Call the function multiple times to ensure deterministic output
        let result1 = sort_and_concatenate_secrets(reveals.clone());
        let result2 = sort_and_concatenate_secrets(reveals.clone());
        
        // Results should be identical (deterministic)
        assert_eq!(result1, result2);
        
        // The secrets should be concatenated in lexicographical order of NodeIds:
        // node1, node2, node3
        let expected: Vec<u8> = [1u8; 32].iter()
            .chain([2u8; 32].iter())
            .chain([3u8; 32].iter())
            .cloned()
            .collect();
        
        assert_eq!(result1, expected);
    }

    #[test]
    fn test_order_matters() {
        // Create reveals in different order but same content
        let mut reveals1 = HashMap::new();
        reveals1.insert("node3".to_string(), RevealPayload {
            round_id: 1,
            secret: [3u8; 32],
        });
        reveals1.insert("node1".to_string(), RevealPayload {
            round_id: 1,
            secret: [1u8; 32],
        });
        reveals1.insert("node2".to_string(), RevealPayload {
            round_id: 1,
            secret: [2u8; 32],
        });

        let mut reveals2 = HashMap::new();
        reveals2.insert("node2".to_string(), RevealPayload {
            round_id: 1,
            secret: [2u8; 32],
        });
        reveals2.insert("node1".to_string(), RevealPayload {
            round_id: 1,
            secret: [1u8; 32],
        });
        reveals2.insert("node3".to_string(), RevealPayload {
            round_id: 1,
            secret: [3u8; 32],
        });

        // Both should produce the same result since they are sorted
        let result1 = sort_and_concatenate_secrets(reveals1);
        let result2 = sort_and_concatenate_secrets(reveals2);
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_single_node() {
        let mut reveals = HashMap::new();
        reveals.insert("node1".to_string(), RevealPayload {
            round_id: 1,
            secret: [42u8; 32],
        });

        let result = sort_and_concatenate_secrets(reveals);
        let expected: Vec<u8> = [42u8; 32].to_vec();
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let reveals = HashMap::new();
        let result = sort_and_concatenate_secrets(reveals);
        
        assert!(result.is_empty());
    }

    #[test]
    fn test_lexicographic_ordering() {
        // Test with NodeIds that demonstrate lexicographic ordering
        let mut reveals = HashMap::new();
        reveals.insert("node10".to_string(), RevealPayload {
            round_id: 1,
            secret: [10u8; 32],
        });
        reveals.insert("node2".to_string(), RevealPayload {
            round_id: 1,
            secret: [2u8; 32],
        });
        reveals.insert("node1".to_string(), RevealPayload {
            round_id: 1,
            secret: [1u8; 32],
        });

        let result = sort_and_concatenate_secrets(reveals);
        
        // Should be ordered as: node1, node10, node2 (lexicographic, not numeric)
        let expected: Vec<u8> = [1u8; 32].iter()
            .chain([10u8; 32].iter())
            .chain([2u8; 32].iter())
            .cloned()
            .collect();
        
        assert_eq!(result, expected);
    }
}