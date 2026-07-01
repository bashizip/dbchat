# Base de test — Boutique

Base de données MySQL/PostgreSQL d'une boutique en ligne, prête à l'emploi avec Docker.

## Démarrage

```bash
docker compose up -d
```

Arrêt :
```bash
docker compose down
```

## Connexion

```bash
# MySQL (port 3306)
dbchat mysql://dbchat:dbchat@localhost:3306/boutique

# Ou directement avec le client MySQL
mysql -u dbchat -pdbchat -h localhost boutique
```

## Schéma

```
┌───────────┐     ┌──────────────┐     ┌───────────┐
│ categories│────>│   produits   │     │  clients  │
└───────────┘     └──────┬───────┘     └─────┬─────┘
                         │                   │
                         │              ┌────┴──────┐
                         │              │ commandes  │
                         │              └────┬───────┘
                         │                   │
                         └──────────┐  ┌─────┴──────────┐
                                    └──│ lignes_commandes│
                                       └────────────────┘
```

### Tables

| Table | Lignes | Description |
|---|---|---|
| `categories` | 8 | Électronique, Vêtements, Maison, Sport, Livres, Alimentation, Jeux, Beauté |
| `produits` | 45 | Nom, prix HT, TVA, stock, catégorie |
| `clients` | 24 | Prénom, nom, email, ville, code postal, date inscription |
| `commandes` | 33 | Client, date, statut (en_attente, validée, expédiée, livrée, annulée) |
| `lignes_commandes` | 64 | Produit, quantité, prix unitaire |

### Colonnes détaillées

**categories**
| Champ | Type | Notes |
|---|---|---|
| id | INT PK | |
| nom | VARCHAR(100) | |
| description | TEXT | |

**produits**
| Champ | Type | Notes |
|---|---|---|
| id | INT PK | |
| nom | VARCHAR(200) | |
| categorie_id | INT FK | → categories.id |
| prix_ht | DECIMAL(10,2) | Prix hors taxes |
| tva | DECIMAL(5,2) | TVA en % (20% ou 5.5%) |
| stock | INT | |
| actif | TINYINT(1) | 1 = en vente |

**clients**
| Champ | Type | Notes |
|---|---|---|
| id | INT PK | |
| prenom | VARCHAR(100) | |
| nom | VARCHAR(100) | |
| email | VARCHAR(200) | UNIQUE |
| ville | VARCHAR(100) | |
| code_postal | VARCHAR(10) | |
| date_inscription | DATE | |

**commandes**
| Champ | Type | Notes |
|---|---|---|
| id | INT PK | |
| client_id | INT FK | → clients.id |
| date_commande | DATETIME | |
| statut | ENUM | en_attente, validée, expédiée, livrée, annulée |

**lignes_commandes**
| Champ | Type | Notes |
|---|---|---|
| id | INT PK | |
| commande_id | INT FK | → commandes.id |
| produit_id | INT FK | → produits.id |
| quantite | INT | |
| prix_unitaire | DECIMAL(10,2) | Prix HT au moment de la vente |

## Questions de test pour dbchat

```sql
-- Top produits les plus vendus
SELECT p.nom, SUM(lc.quantite) as total_vendus
FROM produits p
JOIN lignes_commandes lc ON lc.produit_id = p.id
GROUP BY p.id, p.nom
ORDER BY total_vendus DESC
LIMIT 5;

-- Chiffre d'affaires par catégorie
SELECT c.nom as categorie,
       ROUND(SUM(lc.quantite * lc.prix_unitaire * (1 + p.tva/100)), 2) as ca_ttc
FROM categories c
JOIN produits p ON p.categorie_id = c.id
JOIN lignes_commandes lc ON lc.produit_id = p.id
JOIN commandes cmd ON cmd.id = lc.commande_id
WHERE cmd.statut != 'annulée'
GROUP BY c.id, c.nom
ORDER BY ca_ttc DESC;

-- Clients les plus fidèles
SELECT cl.prenom, cl.nom, cl.ville,
       COUNT(DISTINCT cmd.id) as nb_commandes,
       ROUND(SUM(lc.quantite * lc.prix_unitaire * (1 + p.tva/100)), 2) as total_depense
FROM clients cl
JOIN commandes cmd ON cmd.client_id = cl.id
JOIN lignes_commandes lc ON lc.commande_id = cmd.id
JOIN produits p ON p.id = lc.produit_id
WHERE cmd.statut != 'annulée'
GROUP BY cl.id, cl.prenom, cl.nom, cl.ville
ORDER BY total_depense DESC;

-- Répartition des ventes par mois
SELECT DATE_FORMAT(cmd.date_commande, '%Y-%m') as mois,
       COUNT(DISTINCT cmd.id) as commandes,
       ROUND(SUM(lc.quantite * lc.prix_unitaire * (1 + p.tva/100)), 2) as ca_ttc
FROM commandes cmd
JOIN lignes_commandes lc ON lc.commande_id = cmd.id
JOIN produits p ON p.id = lc.produit_id
WHERE cmd.statut != 'annulée'
GROUP BY mois
ORDER BY mois;

-- Produits en rupture de stock
SELECT nom, stock FROM produits WHERE stock = 0;

-- Panier moyen par client
SELECT cl.prenom, cl.nom,
       ROUND(AVG(panier.total), 2) as panier_moyen
FROM clients cl
JOIN (
    SELECT cmd.client_id, cmd.id,
           SUM(lc.quantite * lc.prix_unitaire * (1 + p.tva/100)) as total
    FROM commandes cmd
    JOIN lignes_commandes lc ON lc.commande_id = cmd.id
    JOIN produits p ON p.id = lc.produit_id
    WHERE cmd.statut != 'annulée'
    GROUP BY cmd.client_id, cmd.id
) panier ON panier.client_id = cl.id
GROUP BY cl.id, cl.prenom, cl.nom
ORDER BY panier_moyen DESC;
```

## Questions en langage naturel

```
dbchat> quels sont les 3 articles les plus vendus ?
dbchat> chiffre d'affaires par catégorie
dbchat> quels clients ont dépensé le plus ?
dbchat> combien de commandes par mois en 2024 ?
dbchat> montre-moi les clients de Paris
dbchat> quel est le panier moyen ?
dbchat> y a-t-il des produits en rupture de stock ?
dbchat> qui s'est inscrit le mois dernier ?
dbchat> répartition des ventes par ville
dbchat> top 5 des produits les plus chers
```
