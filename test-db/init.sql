-- ============================================================
-- Boutique de vente d'articles divers — Jeu de données de test
-- ============================================================

-- ── Catégories ────────────────────────────────────────────
CREATE TABLE categories (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    nom         VARCHAR(100) NOT NULL,
    description TEXT
);

INSERT INTO categories (nom, description) VALUES
    ('Électronique',   'Appareils électroniques, gadgets et accessoires'),
    ('Vêtements',      'Habillement homme, femme et enfant'),
    ('Maison',         'Décoration, cuisine et ameublement'),
    ('Sport',          'Équipement sportif et outdoor'),
    ('Livres',         'Livres, BD et magazines'),
    ('Alimentation',   'Produits alimentaires et boissons'),
    ('Jeux & Jouets',  'Jeux de société, jouets et puzzles'),
    ('Beauté',         'Cosmétiques et soins');

-- ── Produits ──────────────────────────────────────────────
CREATE TABLE produits (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    nom           VARCHAR(200) NOT NULL,
    categorie_id  INT NOT NULL,
    prix_ht       DECIMAL(10,2) NOT NULL,
    tva           DECIMAL(5,2) NOT NULL DEFAULT 20.00,
    stock         INT NOT NULL DEFAULT 0,
    actif         TINYINT(1) NOT NULL DEFAULT 1,
    FOREIGN KEY (categorie_id) REFERENCES categories(id)
);

INSERT INTO produits (nom, categorie_id, prix_ht, tva, stock) VALUES
    -- Électronique
    ('Écouteurs Bluetooth Pro',     1,  49.99,  20.00, 150),
    ('Chargeur USB-C 65W',          1,  29.99,  20.00, 200),
    ('Smartphone X200',             1, 599.00,  20.00,  45),
    ('Tablette T10 128Go',          1, 349.00,  20.00,  30),
    ('Enceinte portable WaterBlast',1,  79.99,  20.00,  80),
    ('Clavier mécanique RGB',       1,  89.99,  20.00,  60),
    ('Souris ergonomique',          1,  39.99,  20.00, 120),
    ('Casque gaming surround',      1, 129.99,  20.00,  40),
    -- Vêtements
    ('T-Shirt coton bio',           2,  19.99,  20.00, 300),
    ('Jean slim noir',              2,  59.99,  20.00, 100),
    ('Veste imperméable',           2,  89.99,  20.00,  50),
    ('Chaussures running',          2,  99.99,  20.00,  70),
    ('Chemise lin blanc',           2,  44.99,  20.00,  90),
    ('Écharpe cachemire',           2,  69.99,  20.00,  35),
    -- Maison
    ('Lampe design LED',            3,  34.99,  20.00,  60),
    ('Coussin décoratif 50x50',     3,  24.99,  20.00, 110),
    ('Machine à café expresso',     3, 199.00,  20.00,  25),
    ('Set de couteaux 5 pièces',    3,  44.99,  20.00,  40),
    ('Bougies parfumées lot 3',     3,  14.99,  20.00, 200),
    ('Cadre photo 20x30cm',         3,   9.99,  20.00, 150),
    -- Sport
    ('Tapis de yoga antidérapant',  4,  29.99,  20.00,  80),
    ('Balle de foot match',         4,  24.99,  20.00,  60),
    ('Sac de sport 45L',            4,  39.99,  20.00,  45),
    ('Raquette de tennis prestige', 4,  89.99,  20.00,  20),
    ('Gourde isotherme 750ml',      4,  19.99,  20.00, 130),
    ('Élastiques de résistance lot',4,  14.99,  20.00, 100),
    -- Livres
    ('Le Guide du Routard France',  5,  14.90,   5.50,  80),
    ('Design Patterns en Rust',     5,  39.99,   5.50,  40),
    ('BD: Tintin au Tibet',         5,  12.99,   5.50, 100),
    ('Roman: Les Misérables',       5,   9.99,   5.50,  60),
    ('Magazine Science & Vie',      5,   6.90,   2.10, 200),
    -- Alimentation
    ('Café en grains 1kg Arabica',  6,  15.99,   5.50, 180),
    ('Thé vert matcha 100g',        6,  12.99,   5.50,  90),
    ('Huile d''olive extra vierge', 6,  11.99,   5.50,  70),
    ('Chocolat noir 85% lot 3',     6,   8.99,   5.50, 150),
    ('Miel de montagne 500g',       6,   9.99,   5.50,  60),
    -- Jeux & Jouets
    ('Puzzle 2000 pièces paysage',  7,  19.99,  20.00,  50),
    ('Jeu de société 7 Wonders',    7,  34.99,  20.00,  30),
    ('Carte Pokémon booster',       7,   4.99,  20.00, 500),
    ('Peluche licorne 40cm',        7,  14.99,  20.00,  80),
    ('Set Lego City police',        7,  49.99,  20.00,  25),
    -- Beauté
    ('Crème hydratante visage',     8,  22.99,  20.00,  90),
    ('Parfum floral 50ml',          8,  45.99,  20.00,  40),
    ('Baume à lèvres bio lot 3',    8,   8.99,  20.00, 200),
    ('Shampoing solide nature',     8,   6.99,  20.00, 120),
    ('Sérum anti-âge vitamine C',   8,  34.99,  20.00,  55);

-- ── Clients ───────────────────────────────────────────────
CREATE TABLE clients (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    prenom      VARCHAR(100) NOT NULL,
    nom         VARCHAR(100) NOT NULL,
    email       VARCHAR(200) NOT NULL UNIQUE,
    ville       VARCHAR(100),
    code_postal VARCHAR(10),
    date_inscription DATE NOT NULL DEFAULT (CURRENT_DATE)
);

INSERT INTO clients (prenom, nom, email, ville, code_postal, date_inscription) VALUES
    ('Sophie',    'Martin',    'sophie.martin@email.fr',      'Paris',         '75001', '2023-01-15'),
    ('Lucas',     'Bernard',   'lucas.bernard@email.fr',      'Lyon',          '69001', '2023-02-20'),
    ('Emma',      'Dubois',    'emma.dubois@email.fr',        'Marseille',     '13001', '2023-03-10'),
    ('Hugo',      'Petit',     'hugo.petit@email.fr',         'Toulouse',      '31000', '2023-04-05'),
    ('Léa',       'Leroy',     'lea.leroy@email.fr',          'Bordeaux',      '33000', '2023-05-12'),
    ('Gabriel',   'Moreau',    'gabriel.moreau@email.fr',     'Lille',         '59000', '2023-06-18'),
    ('Chloé',     'Robert',    'chloe.robert@email.fr',       'Nantes',        '44000', '2023-07-22'),
    ('Adam',      'Richard',   'adam.richard@email.fr',       'Strasbourg',    '67000', '2023-08-30'),
    ('Inès',      'Simon',     'ines.simon@email.fr',         'Montpellier',   '34000', '2023-09-14'),
    ('Nathan',    'Laurent',   'nathan.laurent@email.fr',     'Rennes',        '35000', '2023-10-01'),
    ('Manon',     'Michel',    'manon.michel@email.fr',       'Grenoble',      '38000', '2023-11-11'),
    ('Tom',       'Garcia',    'tom.garcia@email.fr',         'Nice',          '06000', '2023-12-25'),
    ('Jade',      'Fournier',  'jade.fournier@email.fr',      'Angers',        '49000', '2024-01-08'),
    ('Louis',     'Roux',      'louis.roux@email.fr',         'Dijon',         '21000', '2024-01-20'),
    ('Camille',   'Vincent',   'camille.vincent@email.fr',    'Clermont-Fd',   '63000', '2024-02-14'),
    ('Paul',      'Chevalier', 'paul.chevalier@email.fr',     'Le Havre',      '76600', '2024-03-01'),
    ('Sarah',     'Garnier',   'sarah.garnier@email.fr',      'Toulon',        '83000', '2024-03-15'),
    ('Antoine',   'Faure',     'antoine.faure@email.fr',      'Limoges',       '87000', '2024-04-02'),
    ('Zoé',       'Rousseau',  'zoe.rousseau@email.fr',       'Metz',          '57000', '2024-04-18'),
    ('Raphaël',   'Blanc',     'raphael.blanc@email.fr',      'Besançon',      '25000', '2024-05-05'),
    ('Louna',     'Klein',     'louna.klein@email.fr',        'Perpignan',     '66000', '2024-05-22'),
    ('Mathis',    'Brun',      'mathis.brun@email.fr',        'Orléans',       '45000', '2024-06-10'),
    ('Eva',       'Mercier',   'eva.mercier@email.fr',        'Mulhouse',      '68100', '2024-06-28'),
    ('Jules',     'Colin',     'jules.colin@email.fr',        'Caen',          '14000', '2024-07-15');

-- ── Commandes ─────────────────────────────────────────────
CREATE TABLE commandes (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    client_id     INT NOT NULL,
    date_commande DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    statut        ENUM('en_attente', 'validée', 'expédiée', 'livrée', 'annulée') NOT NULL DEFAULT 'en_attente',
    FOREIGN KEY (client_id) REFERENCES clients(id)
);

INSERT INTO commandes (client_id, date_commande, statut) VALUES
    (1,  '2024-01-10 09:30:00', 'livrée'),
    (2,  '2024-01-12 14:15:00', 'livrée'),
    (3,  '2024-01-15 11:00:00', 'livrée'),
    (1,  '2024-01-20 16:45:00', 'livrée'),
    (4,  '2024-02-01 08:20:00', 'livrée'),
    (5,  '2024-02-05 13:30:00', 'livrée'),
    (6,  '2024-02-10 10:00:00', 'livrée'),
    (2,  '2024-02-15 15:10:00', 'livrée'),
    (7,  '2024-02-20 09:45:00', 'expédiée'),
    (8,  '2024-03-01 12:00:00', 'livrée'),
    (9,  '2024-03-05 17:30:00', 'livrée'),
    (10, '2024-03-10 08:15:00', 'livrée'),
    (3,  '2024-03-15 14:00:00', 'livrée'),
    (11, '2024-03-20 10:30:00', 'livrée'),
    (12, '2024-03-25 16:00:00', 'expédiée'),
    (13, '2024-04-01 11:45:00', 'livrée'),
    (14, '2024-04-05 09:00:00', 'livrée'),
    (5,  '2024-04-10 15:30:00', 'livrée'),
    (15, '2024-04-15 12:00:00', 'validée'),
    (16, '2024-04-20 08:45:00', 'livrée'),
    (1,  '2024-05-01 10:00:00', 'livrée'),
    (17, '2024-05-05 14:30:00', 'livrée'),
    (18, '2024-05-10 11:15:00', 'livrée'),
    (19, '2024-05-15 16:00:00', 'expédiée'),
    (20, '2024-05-20 09:30:00', 'livrée'),
    (21, '2024-06-01 13:00:00', 'livrée'),
    (22, '2024-06-05 10:00:00', 'livrée'),
    (23, '2024-06-10 15:45:00', 'livrée'),
    (24, '2024-06-15 08:30:00', 'en_attente'),
    (10, '2024-06-20 12:00:00', 'livrée'),
    (1,  '2024-07-01 14:00:00', 'validée'),
    (6,  '2024-07-05 09:15:00', 'en_attente'),
    (15, '2024-07-10 16:30:00', 'en_attente');

-- ── Lignes de commande ────────────────────────────────────
CREATE TABLE lignes_commandes (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    commande_id INT NOT NULL,
    produit_id  INT NOT NULL,
    quantite    INT NOT NULL,
    prix_unitaire DECIMAL(10,2) NOT NULL,
    FOREIGN KEY (commande_id) REFERENCES commandes(id),
    FOREIGN KEY (produit_id)  REFERENCES produits(id)
);

INSERT INTO lignes_commandes (commande_id, produit_id, quantite, prix_unitaire) VALUES
    (1,  1,  2, 49.99),
    (1,  5,  1, 79.99),
    (2,  9,  3, 19.99),
    (2,  10, 1, 59.99),
    (3,  6,  1, 89.99),
    (3,  7,  2, 39.99),
    (4,  15, 1, 34.99),
    (4,  17, 1, 199.00),
    (4,  19, 2, 14.99),
    (5,  21, 1, 29.99),
    (5,  25, 2, 19.99),
    (6,  3,  1, 599.00),
    (6,  4,  1, 349.00),
    (7,  27, 3, 14.99),
    (7,  28, 1, 39.99),
    (8,  12, 1, 99.99),
    (8,  13, 2, 44.99),
    (9,  30, 5, 6.90),
    (9,  33, 2, 12.99),
    (10, 36, 1, 19.99),
    (10, 37, 1, 34.99),
    (11, 39, 10, 4.99),
    (12, 41, 1, 22.99),
    (12, 42, 1, 45.99),
    (13, 18, 1, 44.99),
    (13, 20, 3, 9.99),
    (14, 23, 1, 39.99),
    (14, 26, 2, 14.99),
    (15, 1,  1, 49.99),
    (15, 8,  1, 129.99),
    (16, 11, 1, 89.99),
    (16, 14, 1, 69.99),
    (17, 2,  3, 29.99),
    (17, 7,  1, 39.99),
    (18, 22, 2, 24.99),
    (18, 24, 1, 89.99),
    (19, 5,  1, 79.99),
    (19, 6,  1, 89.99),
    (20, 29, 2, 12.99),
    (20, 31, 1, 15.99),
    (21, 9,  2, 19.99),
    (21, 10, 1, 59.99),
    (22, 34, 3, 8.99),
    (22, 35, 2, 9.99),
    (23, 38, 1, 49.99),
    (23, 40, 2, 14.99),
    (24, 43, 3, 8.99),
    (24, 44, 2, 6.99),
    (25, 16, 2, 24.99),
    (25, 19, 3, 14.99),
    (26, 32, 2, 12.99),
    (26, 33, 1, 11.99),
    (27, 45, 1, 34.99),
    (28, 11, 1, 89.99),
    (28, 13, 1, 44.99),
    (29, 1,  1, 49.99),
    (29, 17, 1, 199.00),
    (29, 19, 2, 14.99),
    (30, 25, 3, 19.99),
    (30, 26, 1, 14.99),
    (31, 4,  1, 349.00),
    (31, 15, 1, 34.99),
    (31, 20, 2, 9.99),
    (32, 9,  3, 19.99),
    (32, 10, 1, 59.99),
    (33, 37, 1, 34.99),
    (33, 42, 1, 45.99);
